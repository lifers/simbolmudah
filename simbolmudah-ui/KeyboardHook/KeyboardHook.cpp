module;
#include "pch.h"
module KeyboardHook;

import Core;

using namespace winrt;
using namespace Windows::Foundation;

delegate<LowLevelKeyboardEvent> KeyboardHook::Reporter{ nullptr };

LRESULT CALLBACK KeyboardHook::KeyboardProcedure(int nCode, WPARAM wParam, LPARAM lParam)
{
	if (nCode == HC_ACTION)
	{
		const bool is_key = wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN || wParam == WM_KEYUP || wParam == WM_SYSKEYUP;
		const auto keyInfo = reinterpret_cast<const KBDLLHOOKSTRUCT*>(lParam);
		const auto keyEvent = LowLevelKeyboardEvent{
			.vkCode = keyInfo->vkCode,
			.scanCode = keyInfo->scanCode,
			.flags = keyInfo->flags,
			.time = keyInfo->time,
			.dwExtraInfo = keyInfo->dwExtraInfo,
			.windowMessage = wParam
		};
		const bool is_injected = keyInfo->flags & LLKHF_INJECTED;
		if (is_key && !is_injected)
		{
			Reporter(keyEvent);
			return 1;
		}
	}
	return CallNextHookEx(nullptr, nCode, wParam, lParam);
}

KeyboardHook::~KeyboardHook()
{
	MessageBoxW(nullptr, L"Unhooking", L"Info", MB_OK);
	check_bool(UnhookWindowsHookEx(this->m_hook));
	this->m_handle.Cancel();
	Reporter = nullptr;
}

IAsyncAction KeyboardHook::RunAndMonitorListeners()
{
	const auto token{ co_await get_cancellation_token() };
	co_await resume_background();
	const auto hInstance = check_bool(GetModuleHandleW(nullptr));

	// Start listening to keyboard events
	this->m_hook = check_bool(SetWindowsHookExW(WH_KEYBOARD_LL, this->KeyboardProcedure, hInstance, 0));

	while (!token())
	{
		MSG msg;
		if (GetMessageW(&msg, nullptr, 0, 0))
		{
			TranslateMessage(&msg);
			DispatchMessageW(&msg);
		}
	}

	co_return;
}