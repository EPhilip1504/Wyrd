import {
  MemoryRouter as Router,
  Routes,
  Route,
  useNavigate,
  useLocation,
} from "react-router-dom";
import React, { useState, useEffect, useRef, CSSProperties } from "react";
import icon from "../../assets/icons/ezgif-6-cc2e24bb0b-removebg-preview.png";
import "./renderer/styles/App.css";
import WelcomePage from "./renderer/components/WelcomePage";
import LoginSignup, { Form } from "./renderer/components/Signup";
import { invoke } from "@tauri-apps/api/core";
import OTP from "./renderer/components/OTP";
import Personalize from "./renderer/components/Personalize";
export default function App() {
  const [data, setData] = useState("");
  return (
    <Router>
      <Routes>
        {/* Default route: WelcomePage */}
        <Route path="/" element={<WelcomePage />} />

        {/* Route for Login/Signup */}
        <Route path="/signup" element={<Form />} />

        <Route path="/otp" element={<OTP />} />

        <Route path="/personalize" element={<Personalize />} />
      </Routes>
    </Router>
  );
}
