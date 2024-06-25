module;
#include "pch.h"
#include <boost/container/vector.hpp>
#include <winrt/LibSimbolMudah.h>
module KeyboardHook:InputDispatcher;

import ResultSender;

using namespace winrt;
using namespace LibSimbolMudah;
using namespace Windows::Foundation;

namespace {
	constexpr INPUT KeyEventToInput(KBDLLHOOKSTRUCT keyEvent) noexcept
	{
		unsigned long dwFlags = KEYEVENTF_SCANCODE;
		if (keyEvent.flags & LLKHF_EXTENDED)
		{
			dwFlags |= KEYEVENTF_EXTENDEDKEY;
		}
		if (keyEvent.flags & LLKHF_UP)
		{
			dwFlags |= KEYEVENTF_KEYUP;
		}

		return INPUT{
			.type = INPUT_KEYBOARD,
			.ki = KEYBDINPUT{
				.wVk = static_cast<WORD>(keyEvent.vkCode),
				.wScan = static_cast<WORD>(keyEvent.scanCode),
				.dwFlags = dwFlags,
				.time = keyEvent.time,
				.dwExtraInfo = keyEvent.dwExtraInfo
			}
		};
	}

	constexpr std::wstring StageToString(Stage stage) noexcept
	{
		switch (stage)
		{
		case Idle: return L"Idle";
		case ComposeKeydownFirst: return L"ComposeKeydownFirst";
		case ComposeKeyupFirst: return L"ComposeKeyupFirst";
		case ComposeKeydownSecond: return L"ComposeKeydownSecond";
		case SequenceMode: return L"SequenceMode";
		case SearchMode: return L"SearchMode";
		case UnicodeMode: return L"UnicodeMode";
		default: std::unreachable();
		}
	}
}

InputDispatcher::InputDispatcher(
	const winrt::delegate<winrt::fire_and_forget(std::wstring)>& reporterFn,
	winrt::LibSimbolMudah::KeyboardTranslator const& translator
) : m_reporterFn{ reporterFn }, m_keyboardTranslator{ translator }, m_thread{ apartment_context() }
{
	this->m_invalidToken = this->m_keyboardTranslator.OnInvalid(
		TypedEventHandler<KeyboardTranslator, hstring>::TypedEventHandler(this, &InputDispatcher::ResetStage)
	);
}

InputDispatcher::~InputDispatcher()
{
	this->m_keyboardTranslator.OnInvalid(this->m_invalidToken);
}

bool InputDispatcher::ProcessEvent(KBDLLHOOKSTRUCT keyEvent, WPARAM windowMessage)
{
	const bool is_keydown = windowMessage == WM_KEYDOWN || windowMessage == WM_SYSKEYDOWN;
	const uint32_t vkCode = keyEvent.vkCode;
	const uint32_t scanCode = keyEvent.scanCode;

	// Update modifier key states
	switch (vkCode)
	{
	case VK_SHIFT: [[fallthrough]];
	case VK_LSHIFT: [[fallthrough]];
	case VK_RSHIFT:
		this->m_hasShift = is_keydown;
		if (!this->m_hasShift)
			MessageBeep(MB_ICONASTERISK);
		return false;
	case VK_RMENU:
		this->m_hasAltGr = is_keydown;
		break;
	case VK_CAPITAL:
		if (is_keydown)
		{
			this->m_hasCapsLock = !this->m_hasCapsLock;
			return false;
		}
	}

	this->m_inputBuffer.emplace_back(KeyEventToInput(keyEvent));

	switch (this->m_stage)
	{
	case Idle:
		if (is_keydown && vkCode == VK_RMENU)
		{
			this->m_stage = ComposeKeydownFirst;
			this->m_reporterFn(StageToString(this->m_stage));
			return true;
		}
		else
		{
			this->m_inputBuffer.clear();
			this->m_reporterFn(StageToString(this->m_stage));
			return false;
		}
	case ComposeKeydownFirst:
		if (!is_keydown && vkCode == VK_RMENU)
		{
			this->m_stage = ComposeKeyupFirst;
		}
		else
		{
			auto bufferCopy{ this->m_inputBuffer };
			ResultSender::Send(std::span<INPUT>(bufferCopy.data(), bufferCopy.size()));
			this->m_inputBuffer.clear();
			this->m_stage = Idle;
		}
		this->m_reporterFn(StageToString(this->m_stage));
		return true;
	case ComposeKeyupFirst:
		if (is_keydown && vkCode == VK_RMENU)
		{
			this->m_stage = ComposeKeydownSecond;
		}
		else if (is_keydown && vkCode == 0x55) // VK_U
		{
			this->m_stage = UnicodeMode;
			this->m_keyboardTranslator.CheckLayoutAndUpdate();
			this->m_keyboardTranslator.TranslateAndForward(
				vkCode, scanCode, this->m_hasCapsLock, this->m_hasShift, this->m_hasAltGr,
				1
			);
		}
		else
		{
			this->m_stage = SequenceMode;
			this->m_keyboardTranslator.CheckLayoutAndUpdate();
			this->m_keyboardTranslator.TranslateAndForward(
				vkCode, scanCode, this->m_hasCapsLock, this->m_hasShift, this->m_hasAltGr, 0
			);
		}
		this->m_reporterFn(StageToString(this->m_stage));
		return true;
	case ComposeKeydownSecond:
		if (!is_keydown && vkCode == VK_RMENU)
		{
			this->m_stage = SearchMode;
			this->m_inputBuffer.clear();
			this->m_reporterFn(StageToString(this->m_stage));
			// TODO: yield control to search UI
		}
		else
		{
			this->m_stage = SequenceMode;
			this->m_keyboardTranslator.CheckLayoutAndUpdate();
			this->m_keyboardTranslator.TranslateAndForward(
				vkCode, scanCode, this->m_hasCapsLock, this->m_hasShift, this->m_hasAltGr, 0
			);
		}
		this->m_reporterFn(StageToString(this->m_stage));
		return true;
	case SequenceMode:
		if (is_keydown)
		{
			this->m_keyboardTranslator.TranslateAndForward(
				vkCode, scanCode, this->m_hasCapsLock, this->m_hasShift, this->m_hasAltGr, 0
			);
		}
		this->m_reporterFn(StageToString(this->m_stage));
		return true;
	case UnicodeMode:
		if (is_keydown)
		{
			this->m_keyboardTranslator.TranslateAndForward(
				vkCode, scanCode, this->m_hasCapsLock, this->m_hasShift, this->m_hasAltGr, 1
			);
		}
		this->m_reporterFn(StageToString(this->m_stage));
		return true;
	default:
		std::unreachable();
	}
}

winrt::fire_and_forget InputDispatcher::ResetStage(KeyboardTranslator const&, hstring const&)
{
	co_await this->m_thread;
	this->m_inputBuffer.clear();
	this->m_stage = Idle;
	this->m_reporterFn(StageToString(this->m_stage));
}
