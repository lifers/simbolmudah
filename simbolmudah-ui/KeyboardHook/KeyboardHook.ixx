#include "pch.h"
export module KeyboardHook;

import std;
import InputProcessor;

export class KeyboardHook
{
public:
	explicit KeyboardHook(
		const winrt::delegate<winrt::fire_and_forget(KBDLLHOOKSTRUCT, WPARAM)>& reporterFn,
		const winrt::delegate<winrt::fire_and_forget(std::wstring)>& stateFn
	);
	~KeyboardHook();
	KeyboardHook(const KeyboardHook&) = delete;
	KeyboardHook& operator=(const KeyboardHook&) = delete;
	
	const winrt::delegate<winrt::fire_and_forget(KBDLLHOOKSTRUCT, WPARAM)> m_reporterFn;
	bool ProcessEvent(KBDLLHOOKSTRUCT keyEvent, WPARAM windowMessage);

private:
	InputProcessor m_inputProcessor;
	const winrt::Microsoft::UI::Dispatching::DispatcherQueueController m_controller{
		winrt::Microsoft::UI::Dispatching::DispatcherQueueController::CreateOnDedicatedThread()
	};
	HHOOK m_hook{ nullptr };
};
