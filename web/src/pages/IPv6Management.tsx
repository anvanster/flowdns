import React from 'react'
import { Box, Typography } from '@mui/material'

const IPv6Management: React.FC = () => {
  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        IPv6 Management
      </Typography>
      <Typography variant="body1">
        Configure IPv6 settings, SLAAC, and prefix delegation
      </Typography>
    </Box>
  )
}

export default IPv6Management