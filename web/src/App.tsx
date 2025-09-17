import React, { useState, useEffect } from 'react'
import { Routes, Route, Navigate } from 'react-router-dom'
import { Box } from '@mui/material'
import Sidebar from './components/Layout/Sidebar'
import TopBar from './components/Layout/TopBar'
import Dashboard from './pages/Dashboard'
import DhcpLeases from './pages/DhcpLeases'
import DhcpSubnets from './pages/DhcpSubnets'
import DnsZones from './pages/DnsZones'
import DnsRecords from './pages/DnsRecords'
import IPv6Management from './pages/IPv6Management'
import Settings from './pages/Settings'
import Login from './pages/Login'
import { AuthProvider, useAuth } from './contexts/AuthContext'

function AppContent() {
  const { isAuthenticated } = useAuth()
  const [sidebarOpen, setSidebarOpen] = useState(true)

  if (!isAuthenticated) {
    return <Login />
  }

  return (
    <Box sx={{ display: 'flex' }}>
      <Sidebar open={sidebarOpen} onToggle={() => setSidebarOpen(!sidebarOpen)} />
      <Box sx={{ flexGrow: 1, display: 'flex', flexDirection: 'column' }}>
        <TopBar onMenuClick={() => setSidebarOpen(!sidebarOpen)} />
        <Box component="main" sx={{ flexGrow: 1, p: 3 }}>
          <Routes>
            <Route path="/" element={<Navigate to="/dashboard" replace />} />
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/dhcp/leases" element={<DhcpLeases />} />
            <Route path="/dhcp/subnets" element={<DhcpSubnets />} />
            <Route path="/dns/zones" element={<DnsZones />} />
            <Route path="/dns/records" element={<DnsRecords />} />
            <Route path="/ipv6" element={<IPv6Management />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </Box>
      </Box>
    </Box>
  )
}

function App() {
  return (
    <AuthProvider>
      <AppContent />
    </AuthProvider>
  )
}

export default App