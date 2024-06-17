#include "pch.h"
#include <boost/unordered/unordered_flat_map.hpp>
export module KeyboardTranslator;

import std.core;
import :SequenceDictionary;

export class KeyboardTranslator
{
public:
	explicit KeyboardTranslator(
		const winrt::delegate<winrt::fire_and_forget(std::wstring)>& resultReportFn
	) : m_resultReportFn(resultReportFn) {}
	~KeyboardTranslator() = default;
	KeyboardTranslator(const KeyboardTranslator&) = delete;
	KeyboardTranslator& operator=(const KeyboardTranslator&) = delete;

	enum Destination : uint8_t
	{
		Sequence,
		Unicode
	};

	void TranslateAndForward(
		KBDLLHOOKSTRUCT keyEvent, bool hasCapsLock, bool hasShift, bool hasAltGr,
		Destination destination, winrt::delegate<winrt::fire_and_forget()> resetFn
	);
	void CheckLayoutAndUpdate() noexcept;

private:
	struct LiveString : std::wstring { using std::wstring::wstring; };
	struct DeadString : std::wstring { using std::wstring::wstring; };
	struct NoTranslation {};
	typedef std::variant<LiveString, DeadString, NoTranslation> StringVariant;

	HKL m_keyboardLayout{ GetKeyboardLayout(0) };
	const winrt::Microsoft::UI::Dispatching::DispatcherQueueController m_controller{
		winrt::Microsoft::UI::Dispatching::DispatcherQueueController::CreateOnDedicatedThread()
	};

	// give at most 50% load factor
	boost::unordered_flat_map<std::wstring, std::wstring> m_possibleAltGrKeys{
		boost::unordered_flat_map<std::wstring, std::wstring>(2048)
	};
	boost::unordered_flat_map<std::wstring, uint16_t> m_possibleDeadKeys{
		boost::unordered_flat_map<std::wstring, uint16_t>(2048)
	};

	uint16_t m_savedDeadKey{ 0 };
	SequenceDictionary m_sequenceDictionary;
	winrt::delegate<winrt::fire_and_forget(std::wstring)> m_resultReportFn;

	SequenceDictionary::ResultVariant TranslateAndForwardImpl(
		KBDLLHOOKSTRUCT keyEvent, bool hasCapsLock, bool hasShift, bool hasAltGr, Destination destination
	);
	StringVariant VKCodeToUnicode(uint32_t vkCode, uint32_t scanCode, const uint8_t(&keystate)[256], uint32_t flags) const noexcept;
	void ToUnicodeExClearState() const noexcept;
	void AnalyzeLayout() noexcept;
};
