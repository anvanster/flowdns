import React from 'react'
import { Box, Typography } from '@mui/material'

const Settings: React.FC = () => {
  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Settings
      </Typography>
      <Typography variant="body1">
        Configure system settings
      </Typography>
    </Box>
  )
}

export default Settings