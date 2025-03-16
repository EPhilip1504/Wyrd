import React, { useState, useEffect } from "react";
import OtpInput from "react-otp-input";
import axios from "axios";
import { useNavigate, useLocation } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { maskEmail } from "react-email-mask";
import "../styles/OTP.css";

function OTP1() {
  const [
    { otp, numInputs, separator, minLength, maxLength, placeholder, inputType },
    setConfig,
  ] = useState({
    otp: "",
    numInputs: 6,
    separator: "-",
    minLength: 0,
    maxLength: 40,
    placeholder: "",
    inputTypeConf: "isInputNum" as const,
  });

  const OTP_URL = "http://localhost:3000/otp";
  const RESEND_OTP_URL = "http://localhost:3000/resend-otp";
  const navigate = useNavigate();
  const location = useLocation();
  const [isDisabled, setDisabled] = useState(true);

  const handleDisable = () => {
    setDisabled(true);
    setTimeout(() => {
      setDisabled(false);
    }, 3000);
  };

  interface OTP {
    entered_code: string;
  }

  const [isResending, setIsResending] = useState(false);

  const [userEmail, setUserEmail] = useState<string | null>(null);
  let userName;

  useEffect(() => {
    invoke("client_info_otp")
      .then((message: any) => {
        setUserEmail(message.email);
      })
      .catch((error) => console.error(error));
  }, []); // Runs only once when the component mounts

  const onSubmit = async (code: OTP) => {
    try {
      const response = await axios.post(OTP_URL, code, {
        headers: {
          "Content-Type": "application/json",
        },
      });
      console.log("OTP verified successfully:", response.data);
      navigate("/personalize");
    } catch (error) {
      console.error("Error with verification:", error);
      if (axios.isAxiosError(error)) {
        console.error("Axios error:", error.response?.data);
      }
    }
  };

  const handleOTPChange = (otpValue: string): void => {
    setConfig((prevConfig) => ({ ...prevConfig, otp: otpValue }));
    if (otpValue.length === 6) {
      const codeObject: OTP = { entered_code: otpValue };
      onSubmit(codeObject); // Pass the OTP to onSubmit
    }
  };

  const handleResendOTP = async () => {
    try {
      await invoke("resend_otp_handler");
      handleDisable();
    } catch (error) {
      console.error("Error with sending:", error);
    }
  };
  const clearOtp = (): void => {
    setConfig((prevConfig) => ({ ...prevConfig, otp: "" })); // Clear OTP
  };

  const handleSubmit = (event: React.FormEvent<HTMLFormElement>): void => {
    event.preventDefault();
    global.alert(`Submitted OTP: ${otp}`);
  };

  return (
    <div
      className="FormBorder"
      style={{
        backgroundColor: "rgba(255, 255, 255, 0.05)",
        height: "400px",
        width: "550px",
        border: "0.5px solid white",
        borderRadius: "5px",
        display: "flex", // Flexbox for child alignment
        flexDirection: "column", // Ensure the content stacks vertically
        justifyContent: "center", // Center content vertically
        alignItems: "center", // Center content horizontally
        boxShadow: "0px 4px 10px rgba(0, 0, 0, 0.25)", // Optional styling for better visibility
      }}
    >
      <h1
        style={{
          marginBottom: "70px",
        }}
      >
        Verify Email Address
      </h1>

      <p>{`Please enter the OTP we've sent to`}</p>
      <p
        style={{
          marginBottom: "15px",
        }}
      >
        {userEmail ?? "your email"}
      </p>

      <div
        className="inputArea"
        style={{
          display: "flex",
          flexDirection: "column", // Children stack vertically
          alignItems: "center", // Center horizontally
        }}
      >
        <form onSubmit={handleSubmit}>
          <OtpInput
            value={otp} // Bind to correct property
            onChange={handleOTPChange} // Handle changes
            inputType="tel"
            numInputs={numInputs} // Dynamic based on state
            inputStyle={{
              width: "40px",
              height: "50px",
              margin: "5px",
              fontSize: "20px",
              textAlign: "center",
              border: "1px solid hsla(0, 0%, 100%, 0.05)",
              borderRadius: "5px",
              backgroundColor: "rgba(255, 255, 255, 0.05)",
              color: "var(--Text-Loud, var(--Base-White, #fff))",
            }}
            renderInput={(props) => <input {...props} />}
          />
        </form>
      </div>
      <p>
        Didn&apos;t get the code?{""}{" "}
        <span
          onClick={(e) => {
            e.preventDefault();
            handleResendOTP();
          }} //(!isDisabled ? handleResendOTP : e.preventDefault())}
          role="presentation"
          id="resend_button"
        >
          Resend
        </span>
      </p>
    </div>
  );
}

export default OTP1;
