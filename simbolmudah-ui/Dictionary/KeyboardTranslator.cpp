//module;
//#include "pch.h"
//module KeyboardTranslator;
//
//import ResultSender;
//
//namespace {
//	constexpr uint8_t EMPTY_KEYSTATE[256] = {};
//}
//
//void KeyboardTranslator::TranslateAndForward(
//	KBDLLHOOKSTRUCT keyEvent, bool hasCapsLock, bool hasShift, bool hasAltGr,
//	Destination destination, winrt::delegate<winrt::fire_and_forget()> resetFn
//)
//{
//	/* queue the work to this instance's dispatcher:
//	* - translate the key event to a string
//	* - forward the string to the destination, take its return string result
//	* - if the return string result is:
//	*   - incomplete, do nothing
//	*   - complete, print the string using sendinput on another thread
//	*   - invalid, reset InputProcessor's state to Idle
//	*/
//
//	this->m_controller.DispatcherQueue().TryEnqueue([
//		this, keyEvent, hasCapsLock, hasShift, hasAltGr, destination, resetFn
//	](){
//		std::visit([this, resetFn](auto&& result)
//		{
//			using T = std::decay_t<decltype(result)>;
//			if constexpr (std::is_same_v<T, SequenceDictionary::ValidString>)
//			{
//				ResultSender::TranslateAndSend(result);
//				resetFn();
//				this->m_sequenceDictionary.Clear();
//				this->m_resultReportFn(result);
//			}
//			else if constexpr (std::is_same_v<T, SequenceDictionary::Invalid>)
//			{
//				resetFn();
//				this->m_sequenceDictionary.Clear();
//			}
//		}, this->TranslateAndForwardImpl(keyEvent, hasCapsLock, hasShift, hasAltGr, destination));
//	});
//}
//
//SequenceDictionary::ResultVariant KeyboardTranslator::TranslateAndForwardImpl(
//	KBDLLHOOKSTRUCT keyEvent, bool hasCapsLock, bool hasShift, bool hasAltGr, Destination destination
//)
//{
//	uint8_t keystate[256] = {};
//	keystate[VK_CAPITAL] = hasCapsLock ? 0x80 : 0;
//	keystate[VK_SHIFT] = hasShift ? 0x80 : 0;
//	keystate[VK_MENU] = hasAltGr ? 0x80 : 0;
//	keystate[VK_CONTROL] = hasAltGr ? 0x80 : 0;
//
//	return std::visit([this, destination](auto&& arg) -> SequenceDictionary::ResultVariant
//	{
//		using T = std::decay_t<decltype(arg)>;
//		if constexpr (!std::is_same_v<T, NoTranslation>)
//		{
//			return this->m_sequenceDictionary.AppendAndSearch(arg);
//		}
//		else
//		{
//			return SequenceDictionary::Invalid();
//		}
//	}, this->VKCodeToUnicode(keyEvent.vkCode, keyEvent.scanCode, keystate, keyEvent.flags));
//}
//
//KeyboardTranslator::StringVariant KeyboardTranslator::VKCodeToUnicode(
//	uint32_t vkCode, uint32_t scanCode, const uint8_t (&keystate)[256], uint32_t flags
//) const noexcept
//{
//	wchar_t buffer[8];
//	if (int status = ToUnicodeEx(vkCode, scanCode, keystate, buffer, 8, flags, this->m_keyboardLayout); status > 0)
//	{
//		return LiveString{ buffer, static_cast<size_t>(status) };
//	}
//	else if (status < 0)
//	{
//		if (status = ToUnicodeEx(vkCode, scanCode, keystate, buffer, 8, flags, this->m_keyboardLayout); status > 0)
//		{
//			return DeadString{ buffer, static_cast<size_t>(status) };
//		}
//		else
//		{
//			return NoTranslation();
//		}
//	}
//	else
//	{
//		return NoTranslation();
//	}
//}
//
//void KeyboardTranslator::ToUnicodeExClearState() const noexcept
//{
//	this->VKCodeToUnicode(VK_SPACE, 0, EMPTY_KEYSTATE, 0);
//	this->VKCodeToUnicode(VK_SPACE, 0, EMPTY_KEYSTATE, 0);
//}
//
//void KeyboardTranslator::AnalyzeLayout() noexcept
//{
//	this->m_possibleAltGrKeys.clear();
//	this->m_possibleDeadKeys.clear();
//
//	std::wstring noAltGr[256];
//	uint8_t keystate[256] = {};
//
//	for (uint16_t i = 0; i < 0x400; ++i)
//	{
//		const uint16_t vkCode = i & 0xFF;
//		const bool hasShift = (i & 0x100) != 0;
//		const bool hasAltGr = (i & 0x200) != 0;
//
//		keystate[VK_SHIFT] = hasShift ? 0x80 : 0;
//		keystate[VK_MENU] = hasAltGr ? 0x80 : 0;
//		keystate[VK_CONTROL] = hasAltGr ? 0x80 : 0;
//
//		std::visit([this, &noAltGr, vkCode, hasAltGr](auto&& arg)
//		{
//			using T = std::decay_t<decltype(arg)>;
//			if constexpr (!std::is_same_v<T, NoTranslation>)
//			{
//				const auto argString = static_cast<std::wstring>(arg);
//				if (hasAltGr)
//				{
//					this->m_possibleAltGrKeys.insert({ noAltGr[vkCode - 0x200], argString });
//				}
//				else
//				{
//					noAltGr[vkCode] = argString;
//				}
//
//				if constexpr (std::is_same_v<T, DeadString>)
//				{
//					this->m_possibleDeadKeys.insert({ argString, vkCode });
//				}
//			}
//		}, this->VKCodeToUnicode(vkCode, 0, keystate, 0));
//		
//		this->ToUnicodeExClearState();
//	}
//}
//
//void KeyboardTranslator::CheckLayoutAndUpdate() noexcept
//{
//	this->m_controller.DispatcherQueue().TryEnqueue([this]()
//	{
//		const auto foregroundWindow = GetForegroundWindow();
//		const auto tid = GetWindowThreadProcessId(foregroundWindow, nullptr);
//		const auto activeLayout = GetKeyboardLayout(tid);
//
//		if (activeLayout != this->m_keyboardLayout)
//		{
//			this->m_keyboardLayout = activeLayout;
//			this->AnalyzeLayout();
//		}
//	});
//}
