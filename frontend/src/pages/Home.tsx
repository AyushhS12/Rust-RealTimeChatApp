import { useNavigate } from 'react-router-dom';

// --- HELPER SVG ICONS (converted to components for reusability) ---

const MessageSquareIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
    </svg>
);

const ZapIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
    </svg>
);

const LockIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
        <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
    </svg>
);


// --- THE HOME COMPONENT ---

function Home() {
  const navigate = useNavigate()
    const takeToAuth = (mode: string) => {
      navigate("/auth", {state:mode})
    }
    return (
        <div className="bg-gray-50 dark:bg-gray-900 text-gray-800 dark:text-gray-200">
            {/* Header */}
            <header className="fixed w-full bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm z-50 border-b border-gray-200 dark:border-gray-800">
                <div className="container mx-auto px-4 sm:px-6 lg:px-8">
                    <div className="flex items-center justify-between h-16">
                        <div className="flex-shrink-0">
                            <h1 className="text-2xl font-bold gradient-text">Glooo</h1>
                        </div>
                        <nav className="hidden md:flex md:space-x-8">
                            <a href="#features" className="text-gray-500 dark:text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400 transition">Features</a>
                            <a href="#about" className="text-gray-500 dark:text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400 transition">About</a>
                        </nav>
                        <div>
                            <button onClick={()=>takeToAuth("login")} className="hidden sm:inline-block text-sm font-medium text-gray-500 dark:text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400 transition">
                                Log In
                            </button>
                            <button onClick={()=>takeToAuth("signup")} className="ml-4 inline-flex items-center justify-center px-4 py-2 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition">
                                Sign Up
                            </button>
                        </div>
                    </div>
                </div>
            </header>

            <main>
                {/* Hero Section */}
                <section className="pt-32 pb-20 md:pt-40 md:pb-28 text-center">
                    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
                        <h1 className="text-4xl md:text-6xl font-extrabold tracking-tighter mb-6">
                            Connect Instantly with <span className="gradient-text">Glooo</span>
                        </h1>
                        <p className="max-w-2xl mx-auto text-lg text-gray-600 dark:text-gray-400 mb-8">
                            Experience seamless, fast, and secure messaging. Glooo brings you closer to the people who matter most, with a simple and elegant interface.
                        </p>
                        <button onClick={()=>takeToAuth("signup")} className="inline-flex items-center justify-center px-8 py-3 text-lg font-medium text-white bg-indigo-600 border border-transparent rounded-full shadow-lg hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition transform hover:scale-105">
                            Get Started for Free
                        </button>
                    </div>
                </section>

                {/* Features Section */}
                <section id="features" className="py-20 bg-white dark:bg-gray-800">
                    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
                        <div className="text-center mb-12">
                            <h2 className="text-3xl md:text-4xl font-bold">Why You'll Love Glooo</h2>
                            <p className="mt-4 text-lg text-gray-600 dark:text-gray-400">Everything you need in a modern chat app.</p>
                        </div>
                        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
                            {/* Feature 1 */}
                            <div className="p-8 bg-gray-100 dark:bg-gray-700/50 rounded-lg text-center">
                                <div className="flex items-center justify-center h-12 w-12 rounded-full bg-indigo-500 text-white mx-auto mb-4">
                                    <ZapIcon />
                                </div>
                                <h3 className="text-xl font-semibold mb-2">Blazing Fast</h3>
                                <p className="text-gray-600 dark:text-gray-400">Messages are delivered in real-time, instantly. No delays, no waiting.</p>
                            </div>
                            {/* Feature 2 */}
                            <div className="p-8 bg-gray-100 dark:bg-gray-700/50 rounded-lg text-center">
                                <div className="flex items-center justify-center h-12 w-12 rounded-full bg-indigo-500 text-white mx-auto mb-4">
                                    <MessageSquareIcon />
                                </div>
                                <h3 className="text-xl font-semibold mb-2">Rich Conversations</h3>
                                <p className="text-gray-600 dark:text-gray-400">Share text, emojis, and more. Group chats and direct messages, all in one place.</p>
                            </div>
                            {/* Feature 3 */}
                            <div className="p-8 bg-gray-100 dark:bg-gray-700/50 rounded-lg text-center">
                                <div className="flex items-center justify-center h-12 w-12 rounded-full bg-indigo-500 text-white mx-auto mb-4">
                                    <LockIcon />
                                </div>
                                <h3 className="text-xl font-semibold mb-2">Secure & Private</h3>
                                <p className="text-gray-600 dark:text-gray-400">Your conversations are your own. We prioritize your privacy with top-tier security.</p>
                            </div>
                        </div>
                    </div>
                </section>

                {/* About/CTA Section */}
                <section id="about" className="py-20">
                    <div className="container mx-auto px-4 sm:px-6 lg:px-8 text-center">
                        <h2 className="text-3xl md:text-4xl font-bold gradient-text">Ready to Start Talking?</h2>
                        <p className="max-w-2xl mx-auto mt-4 text-lg text-gray-600 dark:text-gray-400 mb-8">
                            Join thousands of users who are already connecting on Glooo. Signing up takes less than a minute.
                        </p>
                        <button onClick={() => takeToAuth("signup")} className="inline-flex items-center justify-center px-8 py-3 text-lg font-medium text-white bg-indigo-600 border border-transparent rounded-full shadow-lg hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition transform hover:scale-105">
                            Create Your Account Now
                        </button>
                    </div>
                </section>
            </main>

            {/* Footer */}
            <footer className="bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700">
                <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-6 text-center text-gray-500 dark:text-gray-400">
                    <p>&copy; {new Date().getFullYear()} Glooo. All rights reserved.</p>
                </div>
            </footer>
        </div>
    );
}

export default Home;

