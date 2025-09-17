import React, { useEffect, useState } from 'react'
import {
  Grid,
  Paper,
  Typography,
  Box,
  Card,
  CardContent,
  CircularProgress,
} from '@mui/material'
import {
  NetworkCheck,
  Dns,
  Router,
  Memory,
} from '@mui/icons-material'
import { Line, Bar } from 'react-chartjs-2'
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  Title,
  Tooltip,
  Legend,
  Filler,
} from 'chart.js'
import axios from 'axios'

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  Title,
  Tooltip,
  Legend,
  Filler
)

interface Metrics {
  dhcp: {
    total_subnets: number
    active_leases: number
    expired_leases: number
    reserved_addresses: number
    available_addresses: number
  }
  dns: {
    total_zones: number
    total_records: number
    dynamic_records: number
  }
  system: {
    uptime_seconds: number
    memory_usage_mb: number
    cpu_usage_percent: number
  }
}

const Dashboard: React.FC = () => {
  const [metrics, setMetrics] = useState<Metrics | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetchMetrics()
    const interval = setInterval(fetchMetrics, 30000) // Refresh every 30 seconds
    return () => clearInterval(interval)
  }, [])

  const fetchMetrics = async () => {
    try {
      const response = await axios.get('/api/v1/system/metrics')
      setMetrics(response.data)
    } catch (error) {
      console.error('Failed to fetch metrics:', error)
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" minHeight="60vh">
        <CircularProgress />
      </Box>
    )
  }

  const statCards = [
    {
      title: 'Active DHCP Leases',
      value: metrics?.dhcp.active_leases || 0,
      icon: <NetworkCheck sx={{ fontSize: 40 }} />,
      color: '#4caf50',
    },
    {
      title: 'DNS Zones',
      value: metrics?.dns.total_zones || 0,
      icon: <Dns sx={{ fontSize: 40 }} />,
      color: '#2196f3',
    },
    {
      title: 'Total Subnets',
      value: metrics?.dhcp.total_subnets || 0,
      icon: <Router sx={{ fontSize: 40 }} />,
      color: '#ff9800',
    },
    {
      title: 'Memory Usage',
      value: `${Math.round(metrics?.system.memory_usage_mb || 0)} MB`,
      icon: <Memory sx={{ fontSize: 40 }} />,
      color: '#9c27b0',
    },
  ]

  const leaseChartData = {
    labels: ['Active', 'Expired', 'Reserved', 'Available'],
    datasets: [
      {
        label: 'DHCP Addresses',
        data: [
          metrics?.dhcp.active_leases || 0,
          metrics?.dhcp.expired_leases || 0,
          metrics?.dhcp.reserved_addresses || 0,
          metrics?.dhcp.available_addresses || 0,
        ],
        backgroundColor: ['#4caf50', '#f44336', '#ff9800', '#2196f3'],
      },
    ],
  }

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Dashboard
      </Typography>

      <Grid container spacing={3}>
        {statCards.map((card, index) => (
          <Grid item xs={12} sm={6} md={3} key={index}>
            <Card>
              <CardContent>
                <Box display="flex" alignItems="center" justifyContent="space-between">
                  <Box>
                    <Typography color="textSecondary" gutterBottom>
                      {card.title}
                    </Typography>
                    <Typography variant="h5">{card.value}</Typography>
                  </Box>
                  <Box sx={{ color: card.color }}>{card.icon}</Box>
                </Box>
              </CardContent>
            </Card>
          </Grid>
        ))}

        <Grid item xs={12} md={8}>
          <Paper sx={{ p: 2 }}>
            <Typography variant="h6" gutterBottom>
              DHCP Address Distribution
            </Typography>
            <Box height={300}>
              <Bar
                data={leaseChartData}
                options={{
                  responsive: true,
                  maintainAspectRatio: false,
                  scales: {
                    y: {
                      beginAtZero: true,
                    },
                  },
                }}
              />
            </Box>
          </Paper>
        </Grid>

        <Grid item xs={12} md={4}>
          <Paper sx={{ p: 2 }}>
            <Typography variant="h6" gutterBottom>
              System Status
            </Typography>
            <Box sx={{ mt: 2 }}>
              <Typography variant="body2" color="textSecondary">
                Uptime
              </Typography>
              <Typography variant="h6">
                {Math.floor((metrics?.system.uptime_seconds || 0) / 3600)} hours
              </Typography>
            </Box>
            <Box sx={{ mt: 2 }}>
              <Typography variant="body2" color="textSecondary">
                CPU Usage
              </Typography>
              <Typography variant="h6">
                {metrics?.system.cpu_usage_percent || 0}%
              </Typography>
            </Box>
            <Box sx={{ mt: 2 }}>
              <Typography variant="body2" color="textSecondary">
                DNS Records
              </Typography>
              <Typography variant="h6">{metrics?.dns.total_records || 0}</Typography>
            </Box>
          </Paper>
        </Grid>
      </Grid>
    </Box>
  )
}

export default Dashboard