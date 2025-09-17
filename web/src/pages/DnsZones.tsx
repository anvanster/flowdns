import React from 'react'
import { Box, Typography } from '@mui/material'

const DnsZones: React.FC = () => {
  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        DNS Zones
      </Typography>
      <Typography variant="body1">
        Manage DNS zones
      </Typography>
    </Box>
  )
}

export default DnsZones