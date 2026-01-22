"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Image from "next/image";
import { useAuth } from "@/app/contexts/AuthContext";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { UserCircle, Shield, AlertCircle } from "lucide-react";
import { toast } from "sonner";

export default function LoginPage() {
    const [selectedRole, setSelectedRole] = useState<"admin" | "agent" | null>(null);
    const [password, setPassword] = useState("");
    const [error, setError] = useState("");
    const { login } = useAuth();
    const router = useRouter();

    const handleRoleSelect = (role: "admin" | "agent") => {
        setSelectedRole(role);
        setError("");
        setPassword("");
    };

    const handleLogin = () => {
        if (!selectedRole) return;

        if (selectedRole === "agent") {
            // Agent doesn't need password
            if (login("agent")) {
                toast.success("Autentificat ca Agent");
                router.push("/");
            }
        } else if (selectedRole === "admin") {
            // Admin needs password
            if (!password) {
                setError("Introduceți parola");
                return;
            }
            if (login("admin", password)) {
                toast.success("Autentificat ca Administrator");
                router.push("/");
            } else {
                setError("Parolă incorectă");
                setPassword("");
            }
        }
    };

    const handleBack = () => {
        setSelectedRole(null);
        setPassword("");
        setError("");
    };

    return (
        <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-primary/5 via-background to-primary/10 p-4">
            <Card className="w-full max-w-md shadow-2xl">
                <CardHeader className="text-center space-y-4">
                    <div className="flex justify-center">
                        <div className="relative h-20 w-20">
                            <Image
                                src="/logo-simbol-transparent.png"
                                alt="eSoft Logo"
                                fill
                                className="object-contain"
                                priority
                            />
                        </div>
                    </div>
                    <div>
                        <CardTitle className="text-2xl font-bold">eSoft Facturi</CardTitle>
                        <CardDescription className="text-base mt-2">
                            {selectedRole ? "Autentificare" : "Selectați tipul de utilizator"}
                        </CardDescription>
                    </div>
                </CardHeader>

                <CardContent className="space-y-4">
                    {!selectedRole ? (
                        // Role selection
                        <div className="space-y-3">
                            <Button
                                onClick={() => handleRoleSelect("agent")}
                                variant="outline"
                                size="lg"
                                className="w-full h-20 text-lg flex items-center gap-3 hover:bg-primary/10 hover:border-primary transition-all"
                            >
                                <UserCircle className="h-8 w-8" />
                                <div className="text-left">
                                    <div className="font-semibold">Agent</div>
                                    <div className="text-xs text-muted-foreground">Acces standard</div>
                                </div>
                            </Button>

                            <Button
                                onClick={() => handleRoleSelect("admin")}
                                variant="outline"
                                size="lg"
                                className="w-full h-20 text-lg flex items-center gap-3 hover:bg-primary/10 hover:border-primary transition-all"
                            >
                                <Shield className="h-8 w-8" />
                                <div className="text-left">
                                    <div className="font-semibold">Administrator</div>
                                    <div className="text-xs text-muted-foreground">Acces complet</div>
                                </div>
                            </Button>
                        </div>
                    ) : (
                        // Login form
                        <div className="space-y-4">
                            <div className="flex items-center gap-3 p-3 bg-muted rounded-lg">
                                {selectedRole === "admin" ? (
                                    <Shield className="h-6 w-6 text-primary" />
                                ) : (
                                    <UserCircle className="h-6 w-6 text-primary" />
                                )}
                                <div>
                                    <div className="font-semibold">
                                        {selectedRole === "admin" ? "Administrator" : "Agent"}
                                    </div>
                                    <div className="text-xs text-muted-foreground">
                                        {selectedRole === "admin" ? "Necesită parolă" : "Fără parolă"}
                                    </div>
                                </div>
                            </div>

                            {selectedRole === "admin" && (
                                <div className="space-y-2">
                                    <Label htmlFor="password">Parolă Administrator</Label>
                                    <Input
                                        id="password"
                                        type="password"
                                        placeholder="Introduceți parola"
                                        value={password}
                                        onChange={(e) => {
                                            setPassword(e.target.value);
                                            setError("");
                                        }}
                                        onKeyDown={(e) => {
                                            if (e.key === "Enter") handleLogin();
                                        }}
                                        autoFocus
                                        className="h-12 text-base"
                                    />
                                    {error && (
                                        <div className="flex items-center gap-2 text-sm text-red-600">
                                            <AlertCircle className="h-4 w-4" />
                                            {error}
                                        </div>
                                    )}
                                </div>
                            )}

                            <div className="flex gap-2">
                                <Button
                                    onClick={handleBack}
                                    variant="outline"
                                    className="flex-1 h-12"
                                >
                                    Înapoi
                                </Button>
                                <Button
                                    onClick={handleLogin}
                                    className="flex-1 h-12"
                                >
                                    {selectedRole === "agent" ? "Continuă" : "Autentificare"}
                                </Button>
                            </div>
                        </div>
                    )}
                </CardContent>
            </Card>
        </div>
    );
}
