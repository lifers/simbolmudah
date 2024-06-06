module;
#include "pch.h"
module KeyboardHook;

using namespace winrt;
using namespace Windows::Foundation;

delegate<DWORD> KeyboardHook::Reporter{ nullptr };

LRESULT CALLBACK KeyboardHook::KeyboardProcedure(int nCode, WPARAM wParam, LPARAM lParam)
{
	if (nCode == HC_ACTION)
	{
		const auto keyInfo = reinterpret_cast<KBDLLHOOKSTRUCT*>(lParam);
		KeyboardHook::Reporter(keyInfo->vkCode);
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