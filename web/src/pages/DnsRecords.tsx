import React from 'react'
import { Box, Typography } from '@mui/material'

const DnsRecords: React.FC = () => {
  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        DNS Records
      </Typography>
      <Typography variant="body1">
        Manage DNS records
      </Typography>
    </Box>
  )
}

export default DnsRecords