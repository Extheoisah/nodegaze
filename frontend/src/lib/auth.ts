import NextAuth from "next-auth";
import CredentialsProvider from "next-auth/providers/credentials";
import type { NextAuthOptions } from "next-auth";

export const authOptions: NextAuthOptions = {
  providers: [
    CredentialsProvider({
      name: "credentials",
      credentials: {
        username: { label: "Username", type: "text" },
        password: { label: "Password", type: "password" },
      },
      async authorize(credentials) {
        if (!credentials?.username || !credentials?.password) {
          console.error("Missing username or password");
          return null;
        }

        try {
          const backendUrl = process.env.BACKEND_URL || "http://localhost:3030";
          const loginResponse = await fetch(`${backendUrl}/auth/login`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              username: credentials.username,
              password: credentials.password,
            }),
          });

          if (!loginResponse.ok) {
            console.error("Login failed:", loginResponse.status);
            return null;
          }

          const loginData = await loginResponse.json();

          if (!loginData.success || !loginData.data.access_token) {
            console.error("Login response invalid:", loginData);
            return null;
          }

          const { access_token, user } = loginData.data;

          if (!access_token || !user) {
            console.error("No access token or user data");
            return null;
          }

          return {
            id: user.id,
            name: user.account_name || user.username,
            email: user.email,
            username: user.username,
            accessToken: access_token,
            role: user.role,
            accountId: user.account_id,
          };
        } catch (error) {
          console.error("Auth error:", error);
          return null;
        }
      },
    }),
  ],
  pages: {
    signIn: "/login",
  },
  callbacks: {
    async jwt({ token, user }) {
      if (user) {
        token.username = user.username || "";
        token.accessToken = user.accessToken;
        token.role = user.role;
        token.accountId = user.accountId;
      }
      return token;
    },
    async session({ session, token }) {
      if (token) {
        session.user.id = token.sub || "";
        session.user.username = token.username || "";
        session.user.role = token.role || "";
        session.user.accountId = token.accountId || "";
        session.accessToken = token.accessToken || "";
      }
      return session;
    },
  },
  session: {
    strategy: "jwt",
  },
  debug: process.env.NODE_ENV === "development",
};

export default NextAuth(authOptions);
