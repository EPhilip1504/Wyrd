import React, { useState } from "react";
import OtpInput from "react-otp-input";
import axios from "axios";
import { useNavigate, useLocation } from "react-router-dom";

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

  const OTP_URL = "http://localhost:1420/otp";
  const RESEND_OTP_URL = "http://localhost:1420/resend-otp";
  const navigate = useNavigate();
  const location = useLocation();
  const userEmail = location.state?.email;
  const userName = location.state?.name as string;
  const [isResending, setIsResending] = useState(false);

  const onSubmit = async (enteredCode: string) => {
    try {
      const response = await axios.post(
        OTP_URL,
        {
          entered_code: String(enteredCode),
          name: userName,
          email: userEmail,
        },
        {
          headers: {
            "Content-Type": "application/json",
          },
        },
      );
      console.log("OTP verified successfully:", response.data);
      navigate("/homepage");
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
      //const codeObject: string = { entered_code: otpValue };
      //onSubmit(codeObject); // Pass the OTP to onSubmit
      console.log(userName);
      console.log(userEmail);
      console.log(otpValue);
    }
  };

  const handleResendOTP = async () => {
    try {
      const response = await axios.post(RESEND_OTP_URL, {
        headers: {
          "Content-Type": "application/json",
        },
      });
      console.log("OTP sent successfully:", response.data);
    } catch (error) {
      console.error("Error with sending:", error);
      if (axios.isAxiosError(error)) {
        console.error("Axios error:", error.response?.data);
      }
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
      <h1>Please enter the OTP sent to {userEmail}</h1>
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
          onClick={(e) => (isResending ? handleResendOTP : e?.preventDefault())}
          role="presentation"
          style={
            isResending
              ? {
                  color: "blue",
                  textDecoration: "underline",
                  cursor: "pointer",
                }
              : { color: "blue", textDecoration: "none" }
          }
        >
          Resend
        </span>
      </p>
    </div>
  );
}

export default OTP1;
