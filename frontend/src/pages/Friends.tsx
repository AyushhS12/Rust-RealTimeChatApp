import { useState, useEffect, useCallback, useRef } from 'react';
import axios from 'axios';
import { Link } from 'react-router-dom';
import useAuth from '../context/useAuth';

// --- TYPE DEFINITIONS ---
interface Id {
  $oid: string;
}

interface SearchedUser {
  _id: Id;
  name: string;
  username: string;
}

interface FriendRequest {
  _id: Id;
  fromUser: {
    _id: Id;
    name: string;
    username: string;
    email: string;
  };
}

const BaseUrl: string = import.meta.env.VITE_BACKEND_URL;

// --- ICONS ---
const SearchIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none"
    stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <circle cx="11" cy="11" r="8" />
    <line x1="21" y1="21" x2="16.65" y2="16.65" />
  </svg>
);

const CheckIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none"
    stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
    <polyline points="20 6 9 17 4 12" />
  </svg>
);

const XIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none"
    stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
    <line x1="18" y1="6" x2="6" y2="18" />
    <line x1="6" y1="6" x2="18" y2="18" />
  </svg>
);

const UserPlusIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none"
    stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
    <circle cx="8.5" cy="7" r="4"></circle>
    <line x1="20" y1="8" x2="20" y2="14"></line>
    <line x1="17" y1="11" x2="23" y2="11"></line>
  </svg>
);

// --- MAIN COMPONENT ---
function Friends() {
  const authGuard = useAuth();
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<SearchedUser[]>([]);
  const [isSearching, setIsSearching] = useState(false);

  const [incomingRequests, setIncomingRequests] = useState<FriendRequest[]>([]);
  const [isLoadingRequests, setIsLoadingRequests] = useState(true);

  const mountedRef = useRef(true);

  // --- FETCH INCOMING REQUESTS ---
  const fetchIncomingRequests = useCallback(async () => {
    setIsLoadingRequests(true);
    try {
      const res = await axios.get(BaseUrl + '/api/requests/get_requests', { withCredentials: true });
      if (mountedRef.current) {
        setIncomingRequests(res.data.requests || []);
      }
    } catch (error) {
      console.error("Failed to fetch incoming friend requests:", error);
    } finally {
      if (mountedRef.current) setIsLoadingRequests(false);
    }
  }, []);
  useEffect(() => {
    mountedRef.current = true;
    authGuard()
    fetchIncomingRequests();

    // cleanup on unmount
    return () => {
      mountedRef.current = false;
    };
  }, [fetchIncomingRequests,authGuard]);

  // --- SEARCH USERS (DEBOUNCED) ---
  useEffect(() => {
    if (searchQuery.trim() === '') {
      setSearchResults([]);
      return;
    }

    setIsSearching(true);
    const timeoutId = setTimeout(async () => {
      try {
        const res = await axios.get(BaseUrl + `/user/search?user=${searchQuery}`, { withCredentials: true });
        if (mountedRef.current) {
          setSearchResults(res.data.users || []);
        }
      } catch (error) {
        console.error("Failed to search for users:", error);
        if (mountedRef.current) setSearchResults([]);
      } finally {
        if (mountedRef.current) setIsSearching(false);
      }
    }, 500);

    return () => clearTimeout(timeoutId);
  }, [searchQuery]);

  // --- HANDLERS ---
  const handleSendRequest = async (userId: string) => {
    try {
      await axios.post(BaseUrl + '/api/requests/send', { to_id: userId }, { withCredentials: true });
      setSearchResults(prev => prev.filter(user => user._id.$oid !== userId));
    } catch (error) {
      console.error("Failed to send friend request:", error);
    }
  };

  const handleAcceptRequest = async (from_id: string) => {
    try {
      await axios.post(BaseUrl + '/api/requests/handle_request',
        { action: "accept", from_id},
        { withCredentials: true });
      setIncomingRequests(prev => prev.filter(req => req._id.$oid !== from_id));
    } catch (error) {
      console.error("Failed to accept friend request:", error);
    }
  };

  const handleRejectRequest = async (from_id: string) => {
    try {
      await axios.post(BaseUrl + '/api/requests/handle_request',
        { action: "reject", from_id},
        { withCredentials: true });
      setIncomingRequests(prev => prev.filter(req => req.fromUser._id.$oid !== from_id));
    } catch (error) {
      console.error("Failed to reject friend request:", error);
    }
  };

  // --- UI ---
  return (
    <div className="min-h-screen bg-gray-100 dark:bg-gray-900 text-gray-800 dark:text-gray-200 p-4 sm:p-6 lg:p-8">
      <div className="max-w-4xl mx-auto">
        {/* Header */}
        <div className="mb-8 flex items-center justify-between">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">Manage Friends</h1>
          <Link to="/chat" className="text-sm font-medium text-indigo-600 hover:text-indigo-500 dark:text-indigo-400 dark:hover:text-indigo-300">
            &larr; Back to Chat
          </Link>
        </div>

        {/* SEARCH USERS */}
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-md mb-8">
          <h2 className="text-xl font-semibold mb-4">Find New Friends</h2>
          <div className="relative">
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <span className="text-gray-500 dark:text-gray-400"><SearchIcon /></span>
            </div>
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search by username..."
              className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-md shadow-sm dark:bg-gray-700 dark:border-gray-600 dark:text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
            />

            {searchQuery && (
              <div className="absolute z-10 w-full mt-2 bg-white dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-md shadow-lg">
                {isSearching ? (
                  <p className="p-4 text-sm text-gray-500">Searching...</p>
                ) : searchResults.length > 0 ? (
                  <ul className="max-h-60 overflow-y-auto">
                    {searchResults.map(user => (
                      <li key={user._id.$oid} className="flex items-center justify-between p-3 hover:bg-gray-50 dark:hover:bg-gray-600">
                        <div>
                          <p className="font-semibold">{user.name}</p>
                          <p className="text-sm text-gray-500 dark:text-gray-400">@{user.username}</p>
                        </div>
                        <button
                          onClick={() => handleSendRequest(user._id.$oid)}
                          className="p-2 text-indigo-600 dark:text-indigo-400 hover:bg-indigo-100 dark:hover:bg-indigo-900/50 rounded-full"
                          title="Send Friend Request"
                        >
                          <UserPlusIcon />
                        </button>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p className="p-4 text-sm text-gray-500">No users found.</p>
                )}
              </div>
            )}
          </div>
        </div>

        {/* INCOMING REQUESTS */}
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-md">
          <h2 className="text-xl font-semibold mb-4">Incoming Friend Requests</h2>
          {isLoadingRequests ? (
            <p>Loading requests...</p>
          ) : incomingRequests.length > 0 ? (
            <div className="space-y-4">
              {incomingRequests.map(req => (
                <div key={req._id.$oid} className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-700/50 rounded-md">
                  <div>
                    <p className="font-semibold">{req.fromUser.name}</p>
                    <p className="text-sm text-gray-500 dark:text-gray-400">@{req.fromUser.username}</p>
                  </div>
                  <div className="flex items-center space-x-2">
                    <button onClick={() => handleAcceptRequest(req.fromUser._id.$oid)} className="p-2 bg-green-500 hover:bg-green-600 text-white rounded-full transition-colors" title="Accept">
                      <CheckIcon />
                    </button>
                    <button onClick={() => handleRejectRequest(req.fromUser._id.$oid)} className="p-2 bg-red-500 hover:bg-red-600 text-white rounded-full transition-colors" title="Reject">
                      <XIcon />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-gray-500 dark:text-gray-400">You have no incoming friend requests.</p>
          )}
        </div>
      </div>
    </div>
  );
}

export default Friends;
