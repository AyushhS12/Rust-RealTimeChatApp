import { createContext, useContext, type Dispatch, type SetStateAction } from "react";

type AuthContextType = {
  token: string;
  setToken: Dispatch<SetStateAction<string>>;
};

export const AuthContext = createContext<AuthContextType | null>(null);



export const useAuthContext = () => {
  const context = useContext(AuthContext);
  if (!context) throw new Error("useAuthContext must be used inside AuthProvider");
  return context;
};
