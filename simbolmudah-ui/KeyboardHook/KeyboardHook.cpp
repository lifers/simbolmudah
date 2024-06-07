module;
#include "pch.h"
module KeyboardHook;

import Core;
import std;

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

bool KeyboardHook::IsHexadecimal(DWORD vkCode)
{
	return false;
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

bool KeyboardHook::ProcessEvent(KBDLLHOOKSTRUCT keyInfo, WPARAM windowMessage)
{
	const bool is_keydown = windowMessage == WM_KEYDOWN || windowMessage == WM_SYSKEYDOWN;

	// Update modifier key states
	switch (keyInfo.vkCode)
	{
	case VK_SHIFT: [[fallthrough]];
	case VK_LSHIFT: [[fallthrough]];
	case VK_RSHIFT:
		m_hasShift = is_keydown;
		return false;
	case VK_RMENU:
		m_hasAltGr = is_keydown;
		break;
	case VK_CAPITAL:
		if (is_keydown)
		{
			m_hasCapsLock = !m_hasCapsLock;
			return false;
		}
	}

	switch (m_stage)
	{
	case Idle:
		if (is_keydown && keyInfo.vkCode == VK_RMENU)
		{
			m_stage = ComposeKeydownFirst;
			return true;
		}
		else
		{
			return false;
		}
	case ComposeKeydownFirst:
		if (!is_keydown && keyInfo.vkCode == VK_RMENU)
		{
			m_stage = ComposeKeyupFirst;
		}
		else
		{
			m_stage = Idle;
		}
		return true;
	case ComposeKeyupFirst:
		if (is_keydown && keyInfo.vkCode == VK_RMENU)
		{
			m_stage = ComposeKeydownSecond;
		}
		else if (is_keydown && keyInfo.vkCode == 0x55) // VK_U
		{
			m_stage = UnicodeMode;
		}
		else
		{
			m_stage = SequenceMode;
			// TODO: append input to buffer
			// TODO: send buffer content to sequencer
		}
		return true;
	case ComposeKeydownSecond:
		if (!is_keydown && keyInfo.vkCode == VK_RMENU)
		{
			m_stage = SearchMode;
			// TODO: yield control to search UI
		}
		else
		{
			m_stage = SequenceMode;
			// TODO: append input to buffer
			// TODO: send buffer content to sequencer
		}
		return true;
	case SequenceMode:
		if (is_keydown)
		{
			// TODO: append input to buffer
			// TODO: send buffer content to sequencer
		}
		return true;
	case UnicodeMode:
		if (is_keydown && IsHexadecimal(keyInfo.vkCode))
		{
			// TODO: append input to buffer
		}
		else if (is_keydown && keyInfo.vkCode == VK_RETURN)
		{
			// TODO: send buffer content to unicode converter
			// TODO: if successful, send the resulting unicode to user
			// TODO: if failed, send and empty the buffer to user
			m_stage = Idle;
		}
		return true;
	default:
		std::unreachable();
	}
}
