import React, { useState, useEffect } from "react";
import { Divider } from "@mui/material";
import "../styles/Personalize.css";

function Personalize() {
  // State to track if dark mode is enabled
  const [isDarkMode, setIsDarkMode] = useState(true);

  // Apply theme when component mounts and when theme changes
  useEffect(() => {
    // Dark mode has blue gradient background
    const darkBackground =
      "linear-gradient(240deg, #12100E 0.09%, #2B4162 90.77%)";
    // Light mode also has gradient background
    const lightBackground =
      "linear-gradient(240deg, #E0EAFC 0.09%, #CFDEF3 90.77%)";

    if (isDarkMode) {
      document.documentElement.setAttribute("data-theme", "dark");
      document.body.style.background = darkBackground;
      document.body.style.color = "white";

      // Update Box1 for dark mode
      const box1Elements = document.getElementsByClassName("Box1");
      if (box1Elements.length > 0) {
        for (let element of box1Elements) {
          element.style.backgroundColor = "rgba(30, 30, 30, 0.85)";
          element.style.color = "white";
          element.style.boxShadow = "0 4px 8px rgba(0, 0, 0, 0.2)";
        }
      }
    } else {
      document.documentElement.setAttribute("data-theme", "light");
      document.body.style.background = lightBackground;
      document.body.style.color = "#333";

      // Update Box1 for light mode
      const box1Elements = document.getElementsByClassName("Box1");
      if (box1Elements.length > 0) {
        for (let element of box1Elements) {
          element.style.backgroundColor = "rgba(240, 240, 240, 0.85)";
          element.style.color = "#333";
          element.style.boxShadow = "0 4px 8px rgba(0, 0, 0, 0.1)";
        }
      }
    }
  }, [isDarkMode]);

  // Function to handle toggle change
  const handleToggleChange = () => {
    setIsDarkMode(!isDarkMode);
  };

  return (
    <div className="Box1">
      <h1>Theme</h1>
      <div className="theme-switch-wrapper">
        <span>Light</span>
        <label className="toggle">
          <input
            type="checkbox"
            checked={isDarkMode}
            onChange={handleToggleChange}
            id="theme-toggle"
          />
          <span className="toggle-slider"></span>
        </label>
        <span>Dark</span>
      </div>
    </div>
  );
}

export default Personalize;
