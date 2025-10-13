"use client";

import { SessionProvider, useSession, signOut } from "next-auth/react";
import { useEffect, useState } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";

/**
 * Component that monitors session for refresh errors and signs out the user
 */
function SessionErrorHandler({ children }: { children: React.ReactNode }) {
  const { data: session } = useSession();

  useEffect(() => {
    if (session?.error === "RefreshAccessTokenError") {
      console.error("Token refresh failed, signing out...");
      signOut({ callbackUrl: "/login" });
    }
  }, [session]);

  return <>{children}</>;
}

export function Providers({ children }: { children: React.ReactNode }) {
  // QueryClient in state to ensure it's only created once per session
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 30000, // Data stays fresh for 30 seconds
            gcTime: 5 * 60 * 1000, // Cache persists for 5 minutes
            retry: 1, // Retry failed requests once
            refetchOnWindowFocus: true, // Refresh when user returns to tab
          },
        },
      })
  );

  return (
    <QueryClientProvider client={queryClient}>
      <SessionProvider>
        <SessionErrorHandler>{children}</SessionErrorHandler>
      </SessionProvider>
      <ReactQueryDevtools initialIsOpen={false} />
    </QueryClientProvider>
  );
}
