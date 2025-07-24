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
        nodePublicKey: { label: "Node Public Key", type: "text" },
        nodeAddress: { label: "Node Address", type: "text" },
        macaroonPath: { label: "Macaroon Path", type: "text" },
        tlsCertPath: { label: "TLS Certificate Path", type: "text" },
      },
      async authorize(credentials) {
        if (!credentials?.username || !credentials?.password) {
          console.error("Missing username or password");
          return null;
        }

        if (
          !credentials?.nodePublicKey ||
          !credentials?.nodeAddress ||
          !credentials?.macaroonPath ||
          !credentials?.tlsCertPath
        ) {
          console.error("Missing node credentials");
          return null;
        }

        try {
          // Step 1: Login to get access token
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

          if (!access_token) {
            console.error("No access token");
            return null;
          }

          const nodeAuthResponse = await fetch(`${backendUrl}/api/node/auth`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
              Authorization: `Bearer ${access_token}`,
            },
            body: JSON.stringify({
              id: credentials.nodePublicKey,
              address: credentials.nodeAddress,
              macaroon: credentials.macaroonPath,
              cert: credentials.tlsCertPath,
            }),
          });

          console.log("Node authentication response:", {
            id: credentials.nodePublicKey,
            address: credentials.nodeAddress,
            macaroon: credentials.macaroonPath,
            cert: credentials.tlsCertPath,
          });

          if (!nodeAuthResponse.ok) {
            console.error(
              "Node authentication response:",
              await nodeAuthResponse.text()
            );
            console.error(
              "Node authentication headers:",
              nodeAuthResponse.headers
            );
            console.error(
              "Node authentication failed:",
              nodeAuthResponse.status
            );
            return null;
          }

          const nodeAuthData = await nodeAuthResponse.json();
          console.log("Node authentication data:", nodeAuthData);

          // Return user data if both authentication steps succeed
          if (user) {
            return {
              id: user.id,
              name: user.account_name || user.username,
              email: user.email,
              username: user.username,
              accessToken: access_token,
              role: user.role,
              accountId: user.account_id,
            };
          }

          return null;
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
