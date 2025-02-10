import React from 'react';

import {
  MemoryRouter as Router,
  Routes,
  Route,
  useNavigate,
  useLocation,
} from 'react-router-dom';

import { Button, Typography, Grid, Paper } from '@mui/material';

function WelcomePage() {
  const navigate = useNavigate(); // Hook for navigation

  return (
    <Grid container justifyContent="center" style={{ marginTop: '2rem' }}>
      <Paper
        style={{ padding: '2rem', textAlign: 'center', maxWidth: '500px' }}
      >
        <Typography variant="h3" gutterBottom>
          Welcome to the App!
        </Typography>
        <Typography variant="body1" paragraph>
          This is your gateway to a world of amazing features. Click below to
          get started.
        </Typography>
        <Button
          variant="contained"
          color="primary"
          onClick={() => navigate('/signup')}
          style={{ marginTop: '1rem' }}
        >
          Get Started
        </Button>
      </Paper>
    </Grid>
  );
}

export default WelcomePage;
