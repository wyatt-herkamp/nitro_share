use std::{fs::File, io::BufReader, path::PathBuf, sync::Arc};

use actix_cors::Cors;
use actix_web::{
    get,
    web::{Data, PayloadConfig},
    App, HttpRequest, HttpServer, Scope,
};
use clap::Parser;
use digestible::Digestible;
use entities::user::database_helpers::first_user;
use helper_macros::{serde_as_rules, Response};
use migration::{Migrator, MigratorTrait};
use nitro_share::{
    admin, config,
    config::{ProfileRules, ServerConfig, SessionConfig, SessionConfigFull, SiteRules},
    images,
    images::ImageRules,
    open_api, paste,
    paste::PasteRules,
    responses::JsonResponse,
    state::State,
    tracing_setup, user,
    user::{
        middleware::HandleSession,
        session::{SessionManager, SessionManagerType},
    },
};
use rustls::{Certificate, PrivateKey, ServerConfig as RustlsServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use sea_orm::{ConnectOptions, SqlxPostgresConnector};
use tracing_actix_web::TracingLogger;
use typeshare::typeshare;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    config: PathBuf,
    /// Rewrites the config filling in any missing values with the default values. Good for updating the config during development
    #[clap(long)]
    rewrite_config: bool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Args = Args::parse();
    let ServerConfig {
        bind_address,
        workers,
        tls,
        database,
        session,
        image_rules,
        paste_rules,
        site_rules,
        profile_rules: public_profiles,
        tracing,
    } = if !args.config.exists() {
        let config = ServerConfig::default();
        let config = toml::to_string(&config)
            .expect("Failed to serialize config. Please report this as a bug.");
        std::fs::write(&args.config, config).expect(&format!("Failed to write config file. Please ensure that the server has write permissions to {:?}",args.config));
        return Ok(());
    } else {
        let config = std::fs::read_to_string(&args.config).unwrap();
        let config: ServerConfig = toml::from_str(&config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        if args.rewrite_config {
            let config = toml::to_string(&config)
                .expect("Failed to serialize config. Please report this as a bug.");
            std::fs::write(&args.config, config).expect(&format!("Failed to write config file. Please ensure that the server has write permissions to {:?}",args.config));
        }
        config
    };
    tracing_setup::setup(tracing).expect("Failed to setup tracing");
    let SessionConfigFull {
        manager,
        session_config,
    } = session;
    let database =
        SqlxPostgresConnector::connect(<config::Database as Into<ConnectOptions>>::into(database))
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Migrator::up(&database, None)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let state = first_user(&database)
        .await
        .map(|u| Data::new(State::new(u)))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let session = SessionManagerType::new(manager, session_config.clone()).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create session manager: {}", e),
        )
    })?;
    let payload_config =
        Data::new(PayloadConfig::default().limit(site_rules.max_payload.get_as_bytes()));
    let database = Data::new(database);
    let session = Data::new(session);
    let openapi = open_api::ApiDoc::openapi();

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method()
            .supports_credentials();

        App::new()
            .app_data(database.clone())
            .app_data(session.clone())
            .app_data(site_rules.clone())
            .app_data(state.clone())
            .app_data(image_rules.clone())
            .app_data(paste_rules.clone())
            .app_data(public_profiles.clone())
            .app_data(payload_config.clone())
            .wrap(TracingLogger::default())
            .wrap(cors)
            .service(
                Scope::new("/api")
                    .wrap(HandleSession {
                        session_manager: session.clone().into_inner(),
                    })
                    .service(site_status)
                    .service(
                        Scope::new("/admin")
                            .configure(admin::init)
                            .service(Scope::new("/user").configure(admin::user::init)),
                    )
                    .service(Scope::new("/user").configure(user::profile::init))
                    .service(Scope::new("/images").configure(images::init))
                    .service(Scope::new("/paste").configure(paste::init))
                    .service(Scope::new("/me").configure(user::me::init))
                    .service(Scope::new("/public").configure(user::public::init)),
            )
            .service(
                Scope::new("/raw")
                    .wrap(HandleSession {
                        session_manager: session.clone().into_inner(),
                    })
                    .service(Scope::new("/paste").configure(paste::init)),
            )
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
    });
    let server = if let Some(workers) = workers {
        server.workers(workers)
    } else {
        server
    };

    let server = if let Some(tls) = tls {
        let mut cert_file = BufReader::new(File::open(tls.certificate_chain)?);
        let mut key_file = BufReader::new(File::open(tls.private_key)?);

        let cert_chain = certs(&mut cert_file)
            .expect("server certificate file error")
            .into_iter()
            .map(Certificate)
            .collect();
        let mut keys: Vec<PrivateKey> = pkcs8_private_keys(&mut key_file)
            .expect("server private key file error")
            .into_iter()
            .map(PrivateKey)
            .collect();

        let config = RustlsServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, keys.remove(0))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        server.bind_rustls_021(bind_address, config)?
    } else {
        server.bind(bind_address)?
    };
    server.run().await?;
    Ok(())
}

#[serde_as_rules]
#[derive(Debug, serde::Serialize, Digestible, Response, ToSchema)]
#[refresh_duration(weeks(1))]
#[typeshare]
pub struct BackendConfigurationReport {
    #[serde_as_rules]
    #[digestible(as_ref = State)]
    #[typeshare(typescript(type = "State"))]
    state: Data<State>,
    #[serde_as_rules]
    #[typeshare(typescript(type = "SessionConfig"))]
    session_config: Arc<SessionConfig>,
    #[serde_as_rules]
    #[digestible(as_ref = SiteRules)]
    #[typeshare(typescript(type = "SiteRules"))]
    site_rules: Data<SiteRules>,
    #[serde_as_rules]
    #[digestible(as_ref = ImageRules)]
    #[typeshare(typescript(type = "ImageRules"))]
    image_rules: Data<ImageRules>,
    #[serde_as_rules]
    #[digestible(as_ref = PasteRules)]
    #[typeshare(typescript(type = "PasteRules"))]
    paste_rules: Data<PasteRules>,
    #[serde_as_rules]
    #[digestible(as_ref = ProfileRules)]
    #[typeshare(typescript(type = "ProfileRules"))]
    public_profiles: Data<ProfileRules>,
}
macro_rules! get_data {
    ($request:ident, $data:ty) => {
        $request
            .app_data::<Data<$data>>()
            .cloned()
            .expect(stringify!("Failed to get data: {}", stringify!($data)))
    };
}
impl BackendConfigurationReport {
    pub fn new(http_request: &HttpRequest) -> Self {
        Self {
            state: get_data!(http_request, State),
            session_config: http_request
                .app_data::<Data<SessionManagerType>>()
                .expect("Failed to get session config")
                .get_session_config(),
            site_rules: get_data!(http_request, SiteRules),
            image_rules: get_data!(http_request, ImageRules),
            paste_rules: get_data!(http_request, PasteRules),
            public_profiles: get_data!(http_request, ProfileRules),
        }
    }
}
#[utoipa::path(get, path = "/api/configuration", responses((status = 200, body = SiteStatusResponse)))]
#[get("/configuration")]
pub async fn site_status(http_request: HttpRequest) -> JsonResponse<BackendConfigurationReport> {
    JsonResponse::from(BackendConfigurationReport::new(&http_request))
}
