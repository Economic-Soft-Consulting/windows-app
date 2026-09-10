"use client";

import { createContext, useContext, useState, useEffect, ReactNode } from "react";

type UserRole = "admin" | "agent" | null;

interface AuthContextType {
    userRole: UserRole;
    isAuthenticated: boolean;
    /**
     * False until the remembered login has been read back from storage. Guards that redirect
     * on `isAuthenticated` must wait for this, or they fire on the first render - when the
     * role is still null - and bounce a remembered agent to the login screen.
     */
    isInitialized: boolean;
    login: (role: UserRole, password?: string) => boolean;
    logout: () => void;
    isAdmin: boolean;
    isAgent: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

const ADMIN_PASSWORD = "esoft2026";
const AGENT_STORAGE_KEY = "agent_logged_in";

export function AuthProvider({ children }: { children: ReactNode }) {
    const [userRole, setUserRole] = useState<UserRole>(null);
    const [isInitialized, setIsInitialized] = useState(false);

    // Check if agent was previously logged in. This cannot be a useState initializer: the app
    // is a static export, so the component is prerendered at build time where there is no
    // localStorage - hence the flag telling consumers the answer is not in yet.
    useEffect(() => {
        const agentLoggedIn = localStorage.getItem(AGENT_STORAGE_KEY);
        if (agentLoggedIn === "true") {
            setUserRole("agent");
        }
        setIsInitialized(true);
    }, []);

    const login = (role: UserRole, password?: string): boolean => {
        if (role === "admin") {
            // Admin requires password
            if (password === ADMIN_PASSWORD) {
                setUserRole("admin");
                // Don't save admin login to localStorage
                return true;
            }
            return false;
        } else if (role === "agent") {
            // Agent doesn't need password
            setUserRole("agent");
            // Remember agent login
            localStorage.setItem(AGENT_STORAGE_KEY, "true");
            return true;
        }
        return false;
    };

    const logout = () => {
        setUserRole(null);
        // Clear agent login from storage
        localStorage.removeItem(AGENT_STORAGE_KEY);
    };

    const value: AuthContextType = {
        userRole,
        isAuthenticated: userRole !== null,
        isInitialized,
        login,
        logout,
        isAdmin: userRole === "admin",
        isAgent: userRole === "agent",
    };

    return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
    const context = useContext(AuthContext);
    if (context === undefined) {
        throw new Error("useAuth must be used within an AuthProvider");
    }
    return context;
}
