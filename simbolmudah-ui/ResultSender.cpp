module;
#include "pch.h"
module ResultSender;

namespace {
	inline void SendInternal(std::span<INPUT> buffer) noexcept
	{
		if (SendInput(buffer.size(), buffer.data(), sizeof(INPUT)) != buffer.size())
		{
			MessageBoxW(nullptr, L"Failed to send input properly", L"Error", MB_OK);
		}
	}
}

winrt::fire_and_forget ResultSender::Send(std::span<INPUT> buffer)
{
	co_await winrt::resume_background();
	SendInternal(buffer);
}

winrt::fire_and_forget ResultSender::TranslateAndSend(std::wstring str)
{
	co_await winrt::resume_background();
	std::vector<INPUT> buffer(str.size() * 2);
	for (const auto& c : str)
	{
		buffer.emplace_back(INPUT {
			.type = INPUT_KEYBOARD,
			.ki = KEYBDINPUT {
				.wVk = 0,
				.wScan = c,
				.dwFlags = KEYEVENTF_UNICODE,
				.time = 0,
				.dwExtraInfo = 0
			}
		});
		buffer.emplace_back(INPUT {
			.type = INPUT_KEYBOARD,
			.ki = KEYBDINPUT {
				.wVk = 0,
				.wScan = c,
				.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
				.time = 0,
				.dwExtraInfo = 0
			}
		});
	}
	SendInternal(buffer);
}