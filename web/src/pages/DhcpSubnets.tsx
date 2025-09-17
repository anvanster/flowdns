import React from 'react'
import { Box, Typography } from '@mui/material'

const DhcpSubnets: React.FC = () => {
  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        DHCP Subnets
      </Typography>
      <Typography variant="body1">
        Manage DHCP subnet configurations
      </Typography>
    </Box>
  )
}

export default DhcpSubnets