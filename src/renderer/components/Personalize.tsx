import React, { useState, useEffect, useRef } from "react";
import {
  Divider,
  Typography,
  Tooltip,
  Switch,
  Slider,
  FormControlLabel,
  RadioGroup,
  Radio,
  FormGroup,
  Checkbox,
} from "@mui/material";
import "../styles/Personalize.css";
import Button from "@mui/material/Button";
import ArrowForwardIcon from "@mui/icons-material/ArrowForward";
import ArrowBackIcon from "@mui/icons-material/ArrowBack";
import BrushIcon from "@mui/icons-material/Brush";
import NotificationsIcon from "@mui/icons-material/Notifications";
import AccessibilityNewIcon from "@mui/icons-material/AccessibilityNew";
import AccountCircleIcon from "@mui/icons-material/AccountCircle";
import SaveIcon from "@mui/icons-material/Save";
import CloudUploadIcon from "@mui/icons-material/CloudUpload";

function Personalize() {
  // State to track if dark mode is enabled
  const [isDarkMode, setIsDarkMode] = useState(true);
  // State to track current personalization step
  const [currentStep, setCurrentStep] = useState(0);
  // State for accent color
  const [accentColor, setAccentColor] = useState("#2196f3");
  // State for font size
  const [fontSize, setFontSize] = useState(16);
  // State for notification settings
  const [notificationSettings, setNotificationSettings] = useState({
    sound: true,
    popup: true,
    doNotDisturb: false,
    weekendOnly: false,
  });
  // State for do not disturb time range
  const [dndTimeRange, setDndTimeRange] = useState({
    start: "22:00",
    end: "07:00",
  });
  // State for accessibility settings
  const [accessibilitySettings, setAccessibilitySettings] = useState({
    highContrast: false,
    reduceMotion: false,
    textToSpeech: false,
  });
  // State for avatar settings
  const [avatarSettings, setAvatarSettings] = useState({
    imageUrl: "",
    useDefault: true,
    defaultAvatarIndex: 0,
  });

  // File input reference
  const fileInputRef = useRef(null);

  // Default avatars
  const defaultAvatars = [
    "https://via.placeholder.com/150/0000FF/FFFFFF?text=1",
    "https://via.placeholder.com/150/FF0000/FFFFFF?text=2",
    "https://via.placeholder.com/150/00FF00/FFFFFF?text=3",
    "https://via.placeholder.com/150/FFFF00/000000?text=4",
  ];

  // Steps for personalization with icons
  const steps = [
    { name: "theme", icon: <BrushIcon /> },
    { name: "notifications", icon: <NotificationsIcon /> },
    { name: "accessibility", icon: <AccessibilityNewIcon /> },
    { name: "avatar", icon: <AccountCircleIcon /> },
  ];

  // Apply theme when component mounts and when theme changes
  useEffect(() => {
    // Dark mode has blue gradient background
    const darkBackground = `linear-gradient(240deg, #12100E 0.09%, #2B4162 90.77%)`;
    // Light mode also has gradient background
    const lightBackground = `linear-gradient(240deg, #E0EAFC 0.09%, #CFDEF3 90.77%)`;

    // Apply accent color to buttons
    document.documentElement.style.setProperty("--accent-color", accentColor);

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
          // Apply dark mode animation
          element.classList.remove("light-mode-border");
          element.classList.add("dark-mode-border");
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
          // Apply light mode animation
          element.classList.remove("dark-mode-border");
          element.classList.add("light-mode-border");
        }
      }
    }
  }, [isDarkMode, accentColor]);

  // Function to handle toggle change
  const handleToggleChange = () => {
    setIsDarkMode(!isDarkMode);
  };

  // Function to handle accent color change
  const handleColorChange = (e) => {
    setAccentColor(e.target.value);
  };

  // Function to handle font size change
  const handleFontSizeChange = (event, newValue) => {
    setFontSize(newValue);
  };

  // Function to handle notification setting change
  const handleNotificationChange = (event) => {
    setNotificationSettings({
      ...notificationSettings,
      [event.target.name]: event.target.checked,
    });
  };

  // Function to handle DND time range change
  const handleDndTimeChange = (event) => {
    setDndTimeRange({
      ...dndTimeRange,
      [event.target.name]: event.target.value,
    });
  };

  // Function to handle accessibility setting change
  const handleAccessibilityChange = (event) => {
    setAccessibilitySettings({
      ...accessibilitySettings,
      [event.target.name]: event.target.checked,
    });
  };

  // Function to handle avatar image upload
  const handleAvatarUpload = (event) => {
    const file = event.target.files[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (e) => {
        setAvatarSettings({
          ...avatarSettings,
          imageUrl: e.target.result,
          useDefault: false,
        });
      };
      reader.readAsDataURL(file);
    }
  };

  // Function to trigger file input click
  const triggerFileInput = () => {
    if (fileInputRef.current) {
      fileInputRef.current.click();
    }
  };

  // Function to select default avatar
  const selectDefaultAvatar = (index) => {
    setAvatarSettings({
      ...avatarSettings,
      defaultAvatarIndex: index,
      useDefault: true,
    });
  };

  // Function to handle next button click
  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);

      // Update URL without full page reload
      window.history.pushState(
        { step: steps[currentStep + 1].name },
        "",
        `/personalize/${steps[currentStep + 1].name}`,
      );
    }
  };

  // Function to handle back button click
  const handleBack = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);

      // Update URL to reflect the previous step
      window.history.pushState(
        { step: steps[currentStep - 1].name },
        "",
        `/personalize/${steps[currentStep - 1].name}`,
      );
    }
  };

  // Function to handle save button click
  const handleSave = () => {
    // Save current step's settings
    const currentStepName = steps[currentStep].name;
    console.log(`Saving ${currentStepName} settings`);

    // Show visual feedback for save
    const saveButton = document.querySelector(".save-button");
    if (saveButton) {
      saveButton.innerText = "Saved!";
      setTimeout(() => {
        saveButton.innerText = "Save";
      }, 1500);
    }

    // Implement actual save logic based on current step
    switch (currentStepName) {
      case "theme":
        console.log(
          `Theme preferences: Dark mode: ${isDarkMode}, Accent color: ${accentColor}, Font size: ${fontSize}px`,
        );
        break;
      case "notifications":
        console.log("Notification settings:", notificationSettings);
        console.log("Do Not Disturb times:", dndTimeRange);
        break;
      case "accessibility":
        console.log("Accessibility settings:", accessibilitySettings);
        break;
      case "avatar":
        console.log("Avatar settings:", avatarSettings);
        break;
      default:
        break;
    }
  };

  // Render step indicator
  const renderStepIndicator = () => {
    return (
      <div className="step-indicator">
        {steps.map((step, index) => (
          <Tooltip
            key={step.name}
            title={step.name.charAt(0).toUpperCase() + step.name.slice(1)}
          >
            <div
              className={`step-dot ${index === currentStep ? "active" : ""}`}
              style={{
                backgroundColor:
                  index === currentStep ? accentColor : undefined,
                boxShadow:
                  index === currentStep ? `0 0 5px ${accentColor}` : undefined,
              }}
            >
              {index <= currentStep && (
                <div className="step-icon">{step.icon}</div>
              )}
            </div>
          </Tooltip>
        ))}
      </div>
    );
  };

  // Render appropriate content based on current step
  const renderStepContent = () => {
    switch (steps[currentStep].name) {
      case "theme":
        return (
          <div className="settings-container theme-settings">
            <Typography variant="h4" className="settings-heading">
              Styles &#038; Theme
            </Typography>
            <Divider className="settings-divider" />

            <div className="setting-group">
              <Typography variant="subtitle1" className="setting-label">
                Color Mode
              </Typography>
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

            <div className="setting-group">
              <Typography variant="subtitle1" className="setting-label">
                Accent Color
              </Typography>

              <div className="color-picker-wrapper">
                <input
                  type="color"
                  id="accent-color"
                  value={accentColor}
                  onChange={handleColorChange}
                  className="color-picker"
                />
                <label htmlFor="accent-color" className="color-picker-label">
                  {accentColor}
                </label>
              </div>
              <div className="color-preview"></div>
            </div>

            <div className="setting-group">
              <Typography variant="subtitle1" className="setting-label">
                Font Size: {fontSize}px
              </Typography>
              <Slider
                value={fontSize}
                min={12}
                max={24}
                step={1}
                onChange={handleFontSizeChange}
                valueLabelDisplay="auto"
                className="font-size-slider"
                style={{
                  color: accentColor,
                }}
              />
              <div
                className="font-size-example"
                style={{ fontSize: `${fontSize}px` }}
              >
                Sample Text
              </div>
            </div>
          </div>
        );
      case "notifications":
        return (
          <div className="settings-container notification-settings">
            <Typography variant="h4" className="settings-heading">
              <NotificationsIcon className="settings-icon" />
              Notification Preferences
            </Typography>
            <Divider className="settings-divider" />

            <div className="setting-group">
              <Typography variant="subtitle1" className="setting-label">
                Chat Notification Style
              </Typography>
              <FormGroup>
                <FormControlLabel
                  control={
                    <Checkbox
                      checked={notificationSettings.sound}
                      onChange={handleNotificationChange}
                      name="sound"
                      style={{ color: accentColor }}
                    />
                  }
                  label="Sound alerts for new messages"
                />
                <FormControlLabel
                  control={
                    <Checkbox
                      checked={notificationSettings.popup}
                      onChange={handleNotificationChange}
                      name="popup"
                      style={{ color: accentColor }}
                    />
                  }
                  label="Pop-up notifications"
                />
              </FormGroup>
            </div>

            <div className="setting-group">
              <Typography variant="subtitle1" className="setting-label">
                Do Not Disturb
              </Typography>
              <FormControlLabel
                control={
                  <Checkbox
                    checked={notificationSettings.doNotDisturb}
                    onChange={handleNotificationChange}
                    name="doNotDisturb"
                    style={{ color: accentColor }}
                  />
                }
                label="Enable Do Not Disturb during specific hours"
              />
              {notificationSettings.doNotDisturb && (
                <div className="time-range-picker">
                  <div className="time-input-container">
                    <label htmlFor="dnd-start">From:</label>
                    <input
                      type="time"
                      id="dnd-start"
                      name="start"
                      value={dndTimeRange.start}
                      onChange={handleDndTimeChange}
                      className="time-input"
                    />
                  </div>
                  <div className="time-input-container">
                    <label htmlFor="dnd-end">To:</label>
                    <input
                      type="time"
                      id="dnd-end"
                      name="end"
                      value={dndTimeRange.end}
                      onChange={handleDndTimeChange}
                      className="time-input"
                    />
                  </div>
                </div>
              )}
              <FormControlLabel
                control={
                  <Checkbox
                    checked={notificationSettings.weekendOnly}
                    onChange={handleNotificationChange}
                    name="weekendOnly"
                    style={{ color: accentColor }}
                  />
                }
                label="Receive notifications only on weekends"
              />
            </div>

            <div className="setting-description">
              <Typography variant="body2">
                These settings control how and when you receive notifications
                from other users. You can change your notification preferences
                anytime from your account settings.
              </Typography>
            </div>
          </div>
        );
      case "accessibility":
        return (
          <div className="settings-container accessibility-settings">
            <Typography variant="h4" className="settings-heading">
              <AccessibilityNewIcon className="settings-icon" />
              Accessibility Features
            </Typography>
            <Divider className="settings-divider" />

            <div className="setting-group">
              <Typography variant="subtitle1" className="setting-label">
                Display Options
              </Typography>
              <FormGroup>
                <FormControlLabel
                  control={
                    <Checkbox
                      checked={accessibilitySettings.highContrast}
                      onChange={handleAccessibilityChange}
                      name="highContrast"
                      style={{ color: accentColor }}
                    />
                  }
                  label="High Contrast Mode"
                />
                <FormControlLabel
                  control={
                    <Checkbox
                      checked={accessibilitySettings.reduceMotion}
                      onChange={handleAccessibilityChange}
                      name="reduceMotion"
                      style={{ color: accentColor }}
                    />
                  }
                  label="Reduce Motion"
                />
              </FormGroup>
            </div>

            <div className="setting-group">
              <Typography variant="subtitle1" className="setting-label">
                Assistive Technologies
              </Typography>
              <FormGroup>
                <FormControlLabel
                  control={
                    <Checkbox
                      checked={accessibilitySettings.textToSpeech}
                      onChange={handleAccessibilityChange}
                      name="textToSpeech"
                      style={{ color: accentColor }}
                    />
                  }
                  label="Enable Text-to-Speech"
                />
              </FormGroup>
            </div>

            <div className="setting-description">
              <Typography variant="body2">
                We're committed to making our product accessible to everyone. If
                you have specific accessibility needs that aren't addressed
                here, please contact our support team.
              </Typography>
            </div>
          </div>
        );
      case "avatar":
        return (
          <div className="settings-container avatar-settings">
            <Typography variant="h4" className="settings-heading">
              <AccountCircleIcon className="settings-icon" />
              Avatar Settings
            </Typography>
            <Divider className="settings-divider" />

            <div className="setting-group">
              <Typography variant="subtitle1" className="setting-label">
                Your Profile Picture
              </Typography>

              <div className="avatar-preview">
                <img
                  src={
                    avatarSettings.useDefault
                      ? defaultAvatars[avatarSettings.defaultAvatarIndex]
                      : avatarSettings.imageUrl ||
                        "https://via.placeholder.com/150/CCCCCC/000000?text=No+Image"
                  }
                  alt="Avatar Preview"
                  className="avatar-image"
                />
              </div>

              <div className="avatar-upload-container">
                <input
                  type="file"
                  accept="image/*"
                  onChange={handleAvatarUpload}
                  style={{ display: "none" }}
                  ref={fileInputRef}
                />
                <Button
                  variant="contained"
                  onClick={triggerFileInput}
                  startIcon={<CloudUploadIcon />}
                  style={{ backgroundColor: accentColor, margin: "10px 0" }}
                >
                  Upload Image
                </Button>
                <Typography variant="body2" className="upload-info">
                  Recommended size: 250x250 pixels. Max file size: 2MB.
                  Supported formats: JPG, PNG, GIF
                </Typography>
              </div>
            </div>

            <div className="setting-group">
              <Typography variant="subtitle1" className="setting-label">
                Default Avatars
              </Typography>
              <div className="default-avatars-container">
                {defaultAvatars.map((avatar, index) => (
                  <div
                    key={index}
                    className={`default-avatar ${avatarSettings.useDefault && avatarSettings.defaultAvatarIndex === index ? "selected" : ""}`}
                    onClick={() => selectDefaultAvatar(index)}
                  >
                    <img src={avatar} alt={`Default Avatar ${index + 1}`} />
                  </div>
                ))}
              </div>
            </div>

            <div className="setting-description">
              <Typography variant="body2">
                Your avatar will be displayed to other users in chat
                conversations and on your profile page.
              </Typography>
            </div>
          </div>
        );
      default:
        return <Typography variant="h4">Theme</Typography>;
    }
  };

  return (
    <div className="Box1">
      {renderStepIndicator()}
      <div className="content-container">{renderStepContent()}</div>

      <div className="button-container">
        {currentStep > 0 && (
          <Button
            variant="outlined"
            className="back-button"
            onClick={handleBack}
            startIcon={<ArrowBackIcon />}
            style={{ borderColor: accentColor, color: accentColor }}
          >
            Back
          </Button>
        )}

        <Button
          variant="contained"
          className="save-button"
          onClick={handleSave}
          startIcon={<SaveIcon />}
          style={{ backgroundColor: accentColor }}
        >
          Save
        </Button>

        {currentStep < steps.length - 1 && (
          <Button
            variant="contained"
            className="next-button"
            onClick={handleNext}
            endIcon={<ArrowForwardIcon />}
            style={{ backgroundColor: accentColor }}
          >
            Next
          </Button>
        )}
      </div>
    </div>
  );
}

export default Personalize;
