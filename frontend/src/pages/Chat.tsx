import axios from "axios";
import { useEffect, useState, useCallback, useRef } from "react";
import useAuth from "../context/useAuth";
import { useLogout } from "../context/useLogout";
import toast from "react-hot-toast";

interface Id {
  $oid: string;
}
interface Message {
  id: Id | null;
  from_id: Id | null;
  to_id: Id;
  content: string;
  chat_id: Id;
}

interface MessageToSend {
  type: string,
  to_id: string;
  content: string;
  chat_id: string;
}
interface Conversation {
  id: Id;
  sender:Id,
  receiver: { id: Id; name: string; username: string };
  last_message: Message;
}

const BaseUrl:string = import.meta.env.VITE_BACKEND_URL;
function Chat() {
  const logout = useLogout();
  const authGuard = useAuth();
  const [id, setid] = useState<Id|null>(null)
  const selectedChatIdRef = useRef<string>("");
  const [chats, setChats] = useState<Conversation[]>([]);
  const [messages, setMessages] = useState<Record<string, Message[]>>({});
  const [selectedChatId, setSelectedChatId] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [input, setInput] = useState("");
  const messagesEndRef = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    axios.get(BaseUrl + "/api/get_my_id").then((val) => {
      setid(val.data)
    }).catch((e) => {
      console.error(e)
    })
  },[])
  const temp = messages[selectedChatId]
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [temp]);

  // 🧠 Reconnection state
  const reconnectAttempts = useRef(0);
  const maxRetries = 5;
  const reconnectTimer = useRef<number | null>(null);
  const connect = useCallback(() => {
    if (reconnectTimer.current) {
      clearTimeout(reconnectTimer.current);
      reconnectTimer.current = null;
    }
    let url;
    if( import.meta.env.VITE_ENV === "production"){
      url = "wss://"+BaseUrl.split("//")[1];
    } else {
      url = "ws://localhost:7878"
    }
    const ws = new WebSocket(`${url}/chat`);

    ws.onopen = () => {
      reconnectAttempts.current = 0;
      setSocket(ws);
    };

    ws.onerror = (e) => {
      console.error("❌ WebSocket error:", e);
      ws.close(1001); // trigger onclose
    };

    ws.onclose = (e) => {
      setSocket(null);
      console.warn("⚠️ WebSocket closed:", e.code, e.reason);

      if (e.code !== 1000) {
        // unexpected close
        if (reconnectAttempts.current < maxRetries) {
          const timeout = Math.min(1000 * 2 ** reconnectAttempts.current, 10000);
          reconnectAttempts.current += 1;
          reconnectTimer.current = setTimeout(() => connect(), timeout);
        } else {
          console.error("🚫 Max reconnection attempts reached.");
          toast.error("Connection lost. Please refresh or try again later.");
        }
      }
    };

    ws.onmessage = (msg) => {
      try {
        const message: Message = JSON.parse(msg.data);
        const chatId = message.chat_id.$oid;
        setMessages((prev) => {
          // Append the message for this chat
          const updated = {
            ...prev,
            [chatId]: [...(prev[chatId] || []), message],
          };
          return updated;
        });
      } catch (err) {
        console.error("Failed to parse incoming message:", err);
      }
    };


    setSocket(ws);
  }, []);

  // 🚀 Fetch chats + connect
  const fetchChatsAndConnect = useCallback(async () => {
    try {
      const res = await axios.get(BaseUrl + "/api/chat/get_chats", {
        withCredentials: true,
      });
      setChats(res.data.chats || []);
      connect();
    } catch (err) {
      console.error("Failed to fetch chats:", err);
    }
  }, [connect]);

  const sendMessage = useCallback(
    (chat: Conversation, msg: string) => {
      const chat_id = chat.id.$oid;
      const message: MessageToSend = {
        type: "direct",
        to_id: chat.receiver.id.$oid,
        content: msg,
        chat_id
      };
      if (socket && socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify(message));
        const realMsg: Message = {
          from_id:id,
          to_id:chat.receiver.id,
          content:msg,
          chat_id:chat.id,
          id:null
        }
        setMessages((prev) => {
          // Append the message for this chat
          const updated = {
            ...prev,
            [chat_id]: [...(prev[chat_id] || []), realMsg],
          };
          return updated;
        });
      } else {
        toast.error("Reconnecting... please try again");
        connect();
      }
    },
    [socket, connect,id]
  );

  const fetchMessages = useCallback(async (chatId: string) => {
    setLoading(true);
    try {
      const res = await axios.get(
        `http://localhost:7878/api/chat/message/get_messages/${chatId}`,
        { withCredentials: true }
      );
      setMessages((prev) => ({
        ...prev,
        [chatId]: res.data.messages || [],
      }));
    } catch (err) {
      console.error("Failed to fetch messages:", err);
    } finally {
      setLoading(false);
    }
  }, []);
  useEffect(() => {
    selectedChatIdRef.current = selectedChatId;
  }, [selectedChatId]);

  useEffect(() => {
    (async () => {
      const ok = await authGuard();
      if (ok) fetchChatsAndConnect();
    })();

    // Cleanup on unmount
    return () => {
      if (socket) socket.close(1000, "Component unmounted");
      if (reconnectTimer.current) clearTimeout(reconnectTimer.current);
    };
  }, [authGuard, fetchChatsAndConnect]);


  const selectedChat = chats.find((c) => c.id.$oid === selectedChatId);
  const chatMessages = messages[selectedChatId] || [];

  return (
    <div className="flex h-screen bg-gray-100 dark:bg-gray-900">
      {/* Left Sidebar — Chat List */}
      <div className="w-1/3 border-r border-gray-300 dark:border-gray-700 flex flex-col">
        <div className="p-4 text-lg font-semibold bg-gray-200 dark:bg-gray-800 dark:text-white">
          Chats
        </div>
        <div className="flex-1 overflow-y-auto divide-y divide-gray-200 dark:divide-gray-700">
          {chats.map((chat) => (
            <div
              key={chat.id.$oid}
              onClick={() => {
                setLoading(true)
                setSelectedChatId(chat.id.$oid)
                fetchMessages(chat.id.$oid)
              }}
              className={`flex items-center p-4 cursor-pointer transition 
              ${selectedChatId === chat.id.$oid
                  ? "bg-gray-300 dark:bg-gray-700"
                  : "hover:bg-gray-200 dark:hover:bg-gray-800"
                }`}
            >
              <div className="h-10 w-10 bg-indigo-500 text-white flex items-center justify-center rounded-full mr-3 font-semibold">
                {chat.receiver.name[0].toUpperCase()}
              </div>
              <div className="flex-1">
                <p className="font-semibold text-gray-800 dark:text-gray-100">
                  {chat.receiver.name}
                </p>
                <p className="text-sm text-gray-500 dark:text-gray-400 truncate">
                  {chat.last_message?.content || "No messages yet"}
                </p>
              </div>
            </div>
          ))}
        </div>
        {!loading && <button
          onClick={logout}
          className="bg-red-500 text-white py-2 mt-auto"
        >
          Logout
        </button>}
      </div>

      {/* Right Side — Chat Window */}
      <div className="flex-1 flex flex-col">
        {selectedChat ? (
          <>
            {/* Chat Header */}
            <div className="p-4 bg-gray-200 dark:bg-gray-800 border-b border-gray-300 dark:border-gray-700">
              <p className="font-semibold text-gray-800 dark:text-gray-100">
                {selectedChat.receiver.name}
              </p>
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-y-auto p-4 space-y-3">
              {chatMessages.map((msg, idx) => (
                <div
                  key={idx}
                  className={`flex ${msg.from_id?.$oid === selectedChat.receiver.id.$oid
                    ? "justify-start"
                    : "justify-end"
                    }`}
                >
                  <div
                    className={`max-w-xs px-4 py-2 rounded-lg text-white ${msg.from_id?.$oid === selectedChat.receiver.id.$oid
                      ? "bg-gray-600"
                      : "bg-indigo-600"
                      }`}
                  >
                    <span className="text-black font-bold">{selectedChat.receiver.id.$oid == msg.from_id?.$oid ? selectedChat.receiver.name : "Me"}</span><br />{msg.content}
                  </div>
                  <div ref={messagesEndRef} />
                </div>
              ))}
            </div>

            {/* Message Input */}
            <div className="p-4 border-t border-gray-300 dark:border-gray-700 flex items-center gap-2">
              <input
                type="text"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                placeholder="Type a message..."
                className="flex-1 p-2 rounded-md bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-white outline-none"
              />
              <button
                onClick={() => {
                  if (input.trim() && selectedChat) {
                    sendMessage(selectedChat, input);
                    setInput("");
                  }
                }}
                className="bg-indigo-600 text-white px-4 py-2 rounded-md hover:bg-indigo-700 transition"
              >
                Send
              </button>
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center text-gray-500 dark:text-gray-400">
            Select a chat to start messaging
          </div>
        )}
      </div>
    </div>
  );
}

export default Chat;
