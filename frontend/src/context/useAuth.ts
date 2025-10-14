import { useCallback } from "react";
import { useNavigate } from "react-router-dom";
import axios from "axios";
import toast from "react-hot-toast";
import { useAuthContext } from "./AuthContext";

const useAuth = (home = false) => {
  const { token, setToken } = useAuthContext();
  const navigate = useNavigate();

  return useCallback(async () => {
    try {
      if (!token) {
        const res = await axios.get(import.meta.env.VITE_BACKEND_URL+"/auth/session", {
          withCredentials: true,
        });
        if (res.data.success) {
          
          if (home) navigate("/chat");
          return true;
        } else {
          toast.error("Please login to continue");
          navigate("/auth");
          return false;
        }
      }
      return true;
    } catch (err) {
      console.error(err)
      toast.error("Session expired, please login again");
      setToken("");
      navigate("/auth");
      return false;
    }
  }, [token, setToken, navigate,home]);
};

export default useAuth;
