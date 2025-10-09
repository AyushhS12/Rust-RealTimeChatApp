import axios from "axios";
import toast from "react-hot-toast";
import { useEffect, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";

const BaseUrl = import.meta.env.BASE_URL;
function Auth() {
  const [isLoginMode, setIsLoginMode] = useState(true);
  const [isLoading, setIsLoading] = useState(false);
  const [form, setForm] = useState({ name: "", username: "", email: "", password: "" });
  // Removed the useContext hook call
  const navigate = useNavigate();
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) =>
    setForm({ ...form, [e.target.id]: e.target.value });

  const toggleMode = () => {
    setIsLoginMode(!isLoginMode);
    setForm({ name: "", username: "", email: "", password: "" });
  };
  const mode = useLocation();
  useEffect(()=>{
  if (mode.state=="login")setIsLoginMode(true); else setIsLoginMode(false);
  },[mode])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    try {
      if (isLoginMode) {
        const res = await axios.post(
          BaseUrl + "/auth/login",
          { email: form.email, password: form.password },
          { withCredentials: true }
        );

        if (res.data.success) {
          // Removed setToken as context is not used here
          toast.success("Welcome back!");
          navigate("/chat");
        } else {
          // Use the error message from the server if available
          toast.error(res.data.message || "Invalid credentials");
        }
      } else {
        await axios.post(BaseUrl + "/auth/signup", form, { withCredentials: true });
        toast.success("Signup successful! Please login.");
        setIsLoginMode(true);
      }
    } catch (err) {
      // Use the error message from the server response if available
      toast.error("Something went wrong");
      console.error(err);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex items-center justify-center min-h-screen bg-gray-100 dark:bg-gray-900 px-4">
      <div className="w-full max-w-md p-8 space-y-6 bg-white rounded-lg shadow-xl dark:bg-gray-800">
        <h2 className="text-3xl font-bold text-center text-gray-900 dark:text-white">
          {isLoginMode ? "Welcome Back!" : "Create an Account"}
        </h2>

        <form className="space-y-6" onSubmit={handleSubmit}>
          {/* --- SIGNUP ONLY FIELDS (with transition) --- */}
          <div
            className={`transition-all duration-500 ease-in-out overflow-hidden ${
              !isLoginMode ? "max-h-60 opacity-100" : "max-h-0 opacity-0"
            }`}
          >
            <div className="space-y-6">
              {/* Full Name Field */}
              <div>
                <label
                  htmlFor="name"
                  className="block text-sm font-medium text-gray-700 dark:text-gray-300"
                >
                  Full Name
                </label>
                <input
                  id="name"
                  type="text"
                  required={!isLoginMode}
                  value={form.name}
                  onChange={handleChange}
                  placeholder="e.g. John Doe"
                  className="w-full px-3 py-2 mt-1 border border-gray-300 rounded-md shadow-sm dark:bg-gray-700 dark:border-gray-600 dark:text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
                />
              </div>
              {/* Username Field */}
              <div>
                <label
                  htmlFor="username"
                  className="block text-sm font-medium text-gray-700 dark:text-gray-300"
                >
                  Username
                </label>
                <input
                  id="username"
                  type="text"
                  required={!isLoginMode}
                  value={form.username}
                  onChange={handleChange}
                  placeholder="e.g. johndoe"
                  className="w-full px-3 py-2 mt-1 border border-gray-300 rounded-md shadow-sm dark:bg-gray-700 dark:border-gray-600 dark:text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
                />
              </div>
            </div>
          </div>

          {/* --- COMMON FIELDS (Email and Password) --- */}
          <div>
            <label
              htmlFor="email"
              className="block text-sm font-medium text-gray-700 dark:text-gray-300"
            >
              Email Address
            </label>
            <input
              id="email"
              type="email"
              autoComplete="email"
              required
              value={form.email}
              onChange={handleChange}
              placeholder="you@example.com"
              className="w-full px-3 py-2 mt-1 border border-gray-300 rounded-md shadow-sm dark:bg-gray-700 dark:border-gray-600 dark:text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
            />
          </div>

          <div>
            <label
              htmlFor="password"
              className="block text-sm font-medium text-gray-700 dark:text-gray-300"
            >
              Password
            </label>
            <input
              id="password"
              type="password"
              autoComplete={isLoginMode ? "current-password" : "new-password"}
              required
              minLength={1}
              value={form.password}
              onChange={handleChange}
              placeholder="••••••••"
              className="w-full px-3 py-2 mt-1 border border-gray-300 rounded-md shadow-sm dark:bg-gray-700 dark:border-gray-600 dark:text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
            />
          </div>

          {/* --- SUBMIT BUTTON --- */}
          <div>
            <button
              type="submit"
              disabled={isLoading}
              className="w-full flex justify-center px-4 py-2 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:bg-indigo-400 disabled:cursor-not-allowed"
            >
              {isLoading ? "Please wait..." : isLoginMode ? "Log In" : "Sign Up"}
            </button>
          </div>
        </form>

        {/* --- TOGGLE MODE LINK --- */}
        <p className="text-sm text-center text-gray-600 dark:text-gray-400">
          {isLoginMode ? "Don't have an account?" : "Already have an account?"}
          <button
            type="button"
            onClick={toggleMode}
            className="ml-1 font-medium text-indigo-600 hover:text-indigo-500 dark:text-indigo-400 dark:hover:text-indigo-300 focus:outline-none"
          >
            {isLoginMode ? "Sign Up" : "Log In"}
          </button>
        </p>
      </div>
    </div>
  );
}

export default Auth;

