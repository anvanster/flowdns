import React from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import {
  Drawer,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Divider,
  Toolbar,
  Typography,
  Box,
} from '@mui/material'
import {
  Dashboard,
  NetworkCheck,
  Dns,
  Router,
  Settings,
  Logout,
  Lan,
  Storage,
} from '@mui/icons-material'
import { useAuth } from '../../contexts/AuthContext'

interface SidebarProps {
  open: boolean
  onToggle: () => void
}

const drawerWidth = 240

const Sidebar: React.FC<SidebarProps> = ({ open }) => {
  const navigate = useNavigate()
  const location = useLocation()
  const { logout } = useAuth()

  const menuItems = [
    { text: 'Dashboard', icon: <Dashboard />, path: '/dashboard' },
    { divider: true },
    { text: 'DHCP Leases', icon: <NetworkCheck />, path: '/dhcp/leases' },
    { text: 'DHCP Subnets', icon: <Router />, path: '/dhcp/subnets' },
    { divider: true },
    { text: 'DNS Zones', icon: <Dns />, path: '/dns/zones' },
    { text: 'DNS Records', icon: <Storage />, path: '/dns/records' },
    { divider: true },
    { text: 'IPv6 Management', icon: <Lan />, path: '/ipv6' },
    { divider: true },
    { text: 'Settings', icon: <Settings />, path: '/settings' },
  ]

  const handleNavigation = (path: string) => {
    navigate(path)
  }

  const handleLogout = () => {
    logout()
    navigate('/login')
  }

  return (
    <Drawer
      sx={{
        width: drawerWidth,
        flexShrink: 0,
        '& .MuiDrawer-paper': {
          width: drawerWidth,
          boxSizing: 'border-box',
        },
      }}
      variant="persistent"
      anchor="left"
      open={open}
    >
      <Toolbar>
        <Typography variant="h6" noWrap component="div">
          FlowDNS
        </Typography>
      </Toolbar>
      <Divider />
      <List>
        {menuItems.map((item, index) => {
          if (item.divider) {
            return <Divider key={index} />
          }
          return (
            <ListItem key={item.text} disablePadding>
              <ListItemButton
                selected={location.pathname === item.path}
                onClick={() => handleNavigation(item.path!)}
              >
                <ListItemIcon>{item.icon}</ListItemIcon>
                <ListItemText primary={item.text} />
              </ListItemButton>
            </ListItem>
          )
        })}
      </List>
      <Box sx={{ flexGrow: 1 }} />
      <Divider />
      <List>
        <ListItem disablePadding>
          <ListItemButton onClick={handleLogout}>
            <ListItemIcon>
              <Logout />
            </ListItemIcon>
            <ListItemText primary="Logout" />
          </ListItemButton>
        </ListItem>
      </List>
    </Drawer>
  )
}

export default Sidebar