import { useCallback } from "react";
import { useNavigate } from "react-router-dom";
import axios from "axios";
import toast from "react-hot-toast";
import { useAuthContext } from "./AuthContext";

export const useLogout = () => {
    const { setToken } = useAuthContext();
    const navigate = useNavigate();

    return useCallback(async () => {
        try {
            await axios.get("http://localhost:7878/auth/logout", {
                withCredentials: true,
            });
        } catch (e) {
            setToken("logged out");
            console.error(e);
        } finally {
            setToken("");
            toast.success("Logged out successfully");
            navigate("/auth");
        }
    }, [setToken, navigate]);
};
