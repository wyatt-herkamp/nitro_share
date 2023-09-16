import http from '@/http'
import { CheckRequest } from '@/types'

export enum CheckStatus {
  Empty = 0,
  Ok = 204,
  Invalid = 400,
  Taken = 409
}

export async function checkParam(check: CheckRequest): Promise<CheckStatus> {
  return await http
    .post(`/api/public/register/check`, check)
    .then((response) => {
      return response.status as CheckStatus
    })
    .catch((error) => {
      if (error.response) {
        return error.response.status as CheckStatus
      }
      console.warn(error)
      return CheckStatus.Invalid
    })
}

export class NewPassword {
  password: string
  password_confirmation: string
  constructor() {
    this.password = ''
    this.password_confirmation = ''
  }

  checkPassword(): boolean {
    if (this.password.length < 8) {
      return false
    }
    return this.password === this.password_confirmation
  }
}
