import NextAuth from "next-auth";
import CredentialsProvider from "next-auth/providers/credentials";
import type { NextAuthOptions } from "next-auth";
import { JWT } from "next-auth/jwt";

/**
 * Refresh the access token using the refresh token
 */
async function refreshAccessToken(token: JWT): Promise<JWT> {
  try {
    const backendUrl = process.env.BACKEND_URL || "http://localhost:3030";
    const response = await fetch(`${backendUrl}/auth/refresh`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ refresh_token: token.refreshToken }),
    });

    if (!response.ok) {
      throw new Error("Refresh token request failed");
    }

    const refreshedData = await response.json();

    if (!refreshedData.success || !refreshedData.data?.access_token) {
      throw new Error("Invalid refresh token response");
    }

    return {
      ...token,
      accessToken: refreshedData.data.access_token,
      accessTokenExpiry: Date.now() + refreshedData.data.expires_in * 1000,
    };
  } catch (error) {
    console.error("Error refreshing access token:", error);
    return {
      ...token,
      error: "RefreshAccessTokenError",
    };
  }
}

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

          const { access_token, refresh_token, user, expires_in } =
            loginData.data;

          if (!access_token || !refresh_token || !user) {
            console.error("No access token, refresh token, or user data");
            return null;
          }

          return {
            id: user.id,
            name: user.account_name || user.username,
            email: user.email,
            username: user.username,
            accessToken: access_token,
            refreshToken: refresh_token,
            expiresIn: expires_in,
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
      // Initial sign in - store user data and tokens
      if (user) {
        token.username = user.username || "";
        token.accessToken = user.accessToken;
        token.refreshToken = user.refreshToken;
        token.expiresIn = user.expiresIn || 3600;
        token.role = user.role;
        token.accountId = user.accountId;
        token.accessTokenExpiry = Date.now() + Number(token.expiresIn) * 1000;
        return token;
      }

      // Token is still valid if its expiry is greater than 5 minutes from now
      if (
        token.accessTokenExpiry &&
        Date.now() < token.accessTokenExpiry - 5 * 60 * 1000
      ) {
        return token;
      }

      console.log("Access token expiring soon, refreshing...");
      return await refreshAccessToken(token);
    },
    async session({ session, token }) {
      if (token) {
        session.user.id = token.sub || "";
        session.user.username = token.username || "";
        session.user.role = token.role || "";
        session.user.accountId = token.accountId || "";
        session.accessToken = token.accessToken || "";
        session.error = token.error;
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
