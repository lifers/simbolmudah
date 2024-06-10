#include "pch.h"
export module KeyboardHook;

import Core;
import InputProcessor;

using namespace winrt;
using namespace Windows::Foundation;
using namespace Microsoft::UI::Dispatching;

export class KeyboardHook
{
public:
	explicit KeyboardHook(const delegate<LowLevelKeyboardEvent>& reporterFn, const delegate<hstring>& stateFn);
	~KeyboardHook();
	
	InputProcessor m_inputProcessor;
	const delegate<LowLevelKeyboardEvent> m_reporterFn;

private:
	const DispatcherQueueController m_controller{ DispatcherQueueController::CreateOnDedicatedThread() };
	HHOOK m_hook{ nullptr };
};
