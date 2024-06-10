module;
#include "pch.h"
module KeyboardHook;

using namespace winrt;
using namespace Windows::Foundation;

namespace {
	KeyboardHook* g_instance = nullptr;

	LRESULT CALLBACK KeyboardProcedure(int nCode, WPARAM wParam, LPARAM lParam) noexcept
	{
		if (nCode == HC_ACTION)
		{
			const bool is_key = wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN || wParam == WM_KEYUP || wParam == WM_SYSKEYUP;
			const auto keyInfo = reinterpret_cast<const KBDLLHOOKSTRUCT*>(lParam);
			const bool is_injected = keyInfo->flags & LLKHF_INJECTED;
			if (is_key && !is_injected)
			{
				g_instance->m_reporterFn(*keyInfo, wParam);
				if (g_instance->ProcessEvent(*keyInfo, wParam))
				{
					return 1;
				}
			}
		}
		return CallNextHookEx(nullptr, nCode, wParam, lParam);
	}
}

KeyboardHook::KeyboardHook(
	const std::function<winrt::fire_and_forget(KBDLLHOOKSTRUCT, WPARAM)>& reporterFn,
	const std::function<winrt::fire_and_forget(std::wstring)>& stateFn
) : m_reporterFn{ reporterFn }, m_inputProcessor{ stateFn }
{
	g_instance = this;

	const auto queue = m_controller.DispatcherQueue();

	const bool successEnqueue = queue.TryEnqueue([this]()
	{
		auto hook = SetWindowsHookExW(WH_KEYBOARD_LL, KeyboardProcedure, GetModuleHandleW(nullptr), 0);
		if (hook)
		{
			this->m_hook = hook;
		}
		else
		{
			MessageBoxW(nullptr, L"Failed to set hook", L"Error", MB_OK);
		}
	});

	if (!successEnqueue)
	{
		MessageBoxW(nullptr, L"Failed to enqueue hook", L"Error", MB_OK);
	}
}

KeyboardHook::~KeyboardHook()
{
	this->m_controller.ShutdownQueueAsync();
	if (UnhookWindowsHookEx(this->m_hook))
	{
		MessageBoxW(nullptr, L"Unhooked successfully", L"Success", MB_OK);
	}
	else
	{
		MessageBoxW(nullptr, std::format(L"Failed to unhook (Error {})", GetLastError()).c_str(), L"Error", MB_OK);
	}
	g_instance = nullptr;
}

bool KeyboardHook::ProcessEvent(KBDLLHOOKSTRUCT keyEvent, WPARAM windowMessage)
{
	return this->m_inputProcessor.ProcessEvent(keyEvent, windowMessage);
}
