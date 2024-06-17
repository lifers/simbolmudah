module;
#include "pch.h"
module KeyboardTranslator:SequenceDictionary;

SequenceDictionary::ResultVariant SequenceDictionary::AppendAndSearch(std::wstring sequence)
{
	this->m_sequence.append_range(sequence);
	if (this->m_sequence == L">")
	{
		return Incomplete();
	}
	else if (this->m_sequence == L">=")
	{
		return ValidString(L"≥");
	}
	else if (this->m_sequence == L"f")
	{
		return Incomplete();
	}
	else if (this->m_sequence == L"fm")
	{
		return ValidString(L"👨🏿‍👩🏻‍👧🏿‍👦🏼");
	}
	else
	{
		return Invalid();
	}
}

void SequenceDictionary::Clear() noexcept
{
	this->m_sequence.clear();
}
