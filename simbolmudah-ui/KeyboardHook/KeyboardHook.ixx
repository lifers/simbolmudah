#include "pch.h"
export module KeyboardHook;

import std;
import InputProcessor;

using namespace winrt;
using namespace Windows::Foundation;
using namespace Microsoft::UI::Dispatching;

export class KeyboardHook
{
public:
	explicit KeyboardHook(
		const std::function<winrt::fire_and_forget(KBDLLHOOKSTRUCT, WPARAM)>& reporterFn,
		const std::function<winrt::fire_and_forget(std::wstring)>& stateFn
	);
	~KeyboardHook();
	
	const std::function<winrt::fire_and_forget(KBDLLHOOKSTRUCT, WPARAM)> m_reporterFn;
	bool ProcessEvent(KBDLLHOOKSTRUCT keyEvent, WPARAM windowMessage);

private:
	InputProcessor m_inputProcessor;
	const DispatcherQueueController m_controller{ DispatcherQueueController::CreateOnDedicatedThread() };
	HHOOK m_hook{ nullptr };
};
