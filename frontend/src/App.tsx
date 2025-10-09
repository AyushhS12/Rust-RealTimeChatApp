// import { useState } from 'react'
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import Home from './pages/Home'
import Auth from './pages/Auth'
import Chat from './pages/Chat'
import { AuthProvider } from './context/AuthContextProvider'
import { Toaster } from 'react-hot-toast'

function App() {

  return (
    <>
    <AuthProvider>
      <Toaster/>
      <BrowserRouter>
        <Routes>
          <Route path='/' element={<Home/>}/>
          <Route path='/auth' element={<Auth/>}/>
            <Route path='/profile' element={<Home/>}/>
          <Route path='/contact' element={<Home/>}/>
          <Route path='/chat' element={<Chat/>}/>
        </Routes>
      </BrowserRouter>
      </AuthProvider>
    </>
  )
}

export default App
