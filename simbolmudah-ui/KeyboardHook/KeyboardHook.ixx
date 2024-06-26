#include "pch.h"
#include <winrt/LibSimbolMudah.h>
#include <winrt/Microsoft.UI.Dispatching.h>
export module KeyboardHook;

import std.core;
import :InputDispatcher;
//import KeyboardTranslator;

export class KeyboardHook
{
public:
	explicit KeyboardHook(
		const winrt::delegate<winrt::fire_and_forget(KBDLLHOOKSTRUCT, WPARAM)> const& reporterFn,
		const winrt::delegate<winrt::fire_and_forget(std::wstring)> const& stateFn,
		const winrt::LibSimbolMudah::KeyboardTranslator const& translator
	);
	~KeyboardHook();
	KeyboardHook(const KeyboardHook&) = delete;
	KeyboardHook& operator=(const KeyboardHook&) = delete;
	
	const winrt::delegate<winrt::fire_and_forget(KBDLLHOOKSTRUCT, WPARAM)> m_reporterFn;
	bool ProcessEvent(KBDLLHOOKSTRUCT keyEvent, WPARAM windowMessage);

private:
	InputDispatcher m_inputProcessor;
	const winrt::Microsoft::UI::Dispatching::DispatcherQueueController m_controller{
		winrt::Microsoft::UI::Dispatching::DispatcherQueueController::CreateOnDedicatedThread()
	};
	HHOOK m_hook{ nullptr };
};
