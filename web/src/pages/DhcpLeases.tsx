import React, { useState, useEffect } from 'react'
import {
  Box,
  Paper,
  Typography,
  Button,
  IconButton,
  Chip,
  TextField,
  InputAdornment,
} from '@mui/material'
import { DataGrid, GridColDef, GridRenderCellParams } from '@mui/x-data-grid'
import {
  Refresh,
  Delete,
  Search,
  Download,
} from '@mui/icons-material'
import axios from 'axios'
import { format } from 'date-fns'

interface Lease {
  id: string
  mac_address: string
  ip_address: string
  hostname: string | null
  lease_start: string
  lease_end: string
  state: string
  subnet_name?: string
}

const DhcpLeases: React.FC = () => {
  const [leases, setLeases] = useState<Lease[]>([])
  const [loading, setLoading] = useState(true)
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedRows, setSelectedRows] = useState<string[]>([])

  useEffect(() => {
    fetchLeases()
  }, [])

  const fetchLeases = async () => {
    setLoading(true)
    try {
      const response = await axios.get('/api/v1/dhcp/leases')
      setLeases(response.data)
    } catch (error) {
      console.error('Failed to fetch leases:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleReleaseLease = async (leaseId: string) => {
    try {
      await axios.delete(`/api/v1/dhcp/leases/${leaseId}`)
      fetchLeases()
    } catch (error) {
      console.error('Failed to release lease:', error)
    }
  }

  const handleBulkRelease = async () => {
    for (const leaseId of selectedRows) {
      await handleReleaseLease(leaseId)
    }
    setSelectedRows([])
  }

  const exportLeases = () => {
    const csv = [
      ['MAC Address', 'IP Address', 'Hostname', 'Lease Start', 'Lease End', 'State'],
      ...leases.map(lease => [
        lease.mac_address,
        lease.ip_address,
        lease.hostname || '',
        lease.lease_start,
        lease.lease_end,
        lease.state,
      ]),
    ]
      .map(row => row.join(','))
      .join('\n')

    const blob = new Blob([csv], { type: 'text/csv' })
    const url = window.URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `dhcp-leases-${format(new Date(), 'yyyy-MM-dd')}.csv`
    a.click()
  }

  const columns: GridColDef[] = [
    {
      field: 'mac_address',
      headerName: 'MAC Address',
      width: 150,
      renderCell: (params) => (
        <Typography sx={{ fontFamily: 'monospace' }}>
          {params.value}
        </Typography>
      ),
    },
    {
      field: 'ip_address',
      headerName: 'IP Address',
      width: 130,
      renderCell: (params) => (
        <Typography sx={{ fontFamily: 'monospace' }}>
          {params.value}
        </Typography>
      ),
    },
    {
      field: 'hostname',
      headerName: 'Hostname',
      width: 200,
      renderCell: (params) => params.value || '-',
    },
    {
      field: 'state',
      headerName: 'State',
      width: 100,
      renderCell: (params: GridRenderCellParams) => {
        const color =
          params.value === 'active'
            ? 'success'
            : params.value === 'expired'
            ? 'error'
            : 'default'
        return <Chip label={params.value} color={color} size="small" />
      },
    },
    {
      field: 'lease_start',
      headerName: 'Lease Start',
      width: 180,
      renderCell: (params) =>
        format(new Date(params.value), 'yyyy-MM-dd HH:mm:ss'),
    },
    {
      field: 'lease_end',
      headerName: 'Lease End',
      width: 180,
      renderCell: (params) =>
        format(new Date(params.value), 'yyyy-MM-dd HH:mm:ss'),
    },
    {
      field: 'actions',
      headerName: 'Actions',
      width: 100,
      sortable: false,
      renderCell: (params: GridRenderCellParams) => (
        <IconButton
          size="small"
          onClick={() => handleReleaseLease(params.row.id)}
          disabled={params.row.state !== 'active'}
        >
          <Delete />
        </IconButton>
      ),
    },
  ]

  const filteredLeases = leases.filter(
    (lease) =>
      lease.mac_address.toLowerCase().includes(searchTerm.toLowerCase()) ||
      lease.ip_address.includes(searchTerm) ||
      (lease.hostname?.toLowerCase().includes(searchTerm.toLowerCase()) || false)
  )

  return (
    <Box>
      <Box display="flex" justifyContent="space-between" alignItems="center" mb={3}>
        <Typography variant="h4">DHCP Leases</Typography>
        <Box display="flex" gap={2}>
          <Button
            variant="outlined"
            startIcon={<Download />}
            onClick={exportLeases}
          >
            Export CSV
          </Button>
          <Button
            variant="contained"
            startIcon={<Refresh />}
            onClick={fetchLeases}
          >
            Refresh
          </Button>
        </Box>
      </Box>

      <Paper sx={{ p: 2, mb: 2 }}>
        <Box display="flex" gap={2} alignItems="center">
          <TextField
            placeholder="Search by MAC, IP, or hostname..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            size="small"
            sx={{ flexGrow: 1, maxWidth: 400 }}
            InputProps={{
              startAdornment: (
                <InputAdornment position="start">
                  <Search />
                </InputAdornment>
              ),
            }}
          />
          {selectedRows.length > 0 && (
            <Button
              variant="contained"
              color="error"
              startIcon={<Delete />}
              onClick={handleBulkRelease}
            >
              Release {selectedRows.length} Leases
            </Button>
          )}
        </Box>
      </Paper>

      <Paper sx={{ height: 600 }}>
        <DataGrid
          rows={filteredLeases}
          columns={columns}
          loading={loading}
          checkboxSelection
          onRowSelectionModelChange={(newSelection) => {
            setSelectedRows(newSelection as string[])
          }}
          pageSizeOptions={[10, 25, 50, 100]}
          initialState={{
            pagination: {
              paginationModel: { pageSize: 25 },
            },
          }}
        />
      </Paper>
    </Box>
  )
}

export default DhcpLeases