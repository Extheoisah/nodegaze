import NextAuth from "next-auth"

declare module "next-auth" {
  interface Session {
    user: {
      id: string
      name?: string | null
      email?: string | null
      image?: string | null
      username?: string
      role?: string
      accountId?: string
    }
    accessToken?: string
  }

  interface User {
    id: string
    username?: string
    accessToken?: string
    role?: string
    accountId?: string
  }
}

declare module "next-auth/jwt" {
  interface JWT {
    username?: string
    accessToken?: string
    role?: string
    accountId?: string
  }
} 