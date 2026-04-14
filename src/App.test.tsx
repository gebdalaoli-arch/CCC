import { render, screen } from "@testing-library/react";
import App from "./App";

describe("Codex Launcher shell", () => {
  it("renders the launcher title", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "Codex Launcher" })).toBeInTheDocument();
  });

  it("disables save-and-launch without Base URL/API Key and shows compatibility note", () => {
    render(<App />);

    const launchButton = screen.getByRole("button", { name: "保存并启动" });
    expect(launchButton).toBeDisabled();
    expect(
      screen.getByText(/会清理 subscription tokens 与历史订阅登录残留/i)
    ).toBeInTheDocument();
    expect(
      screen.getByText(/归一化 OpenAI\/openai provider 标签/i)
    ).toBeInTheDocument();
  });
});
