// Re-export from the singleton context so all components share the same instance.
// The actual auto-send logic runs only in OnlineStatusProvider (mounted once in layout).
export { useOnlineStatus } from "@/app/contexts/OnlineStatusContext";
