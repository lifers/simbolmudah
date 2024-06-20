﻿#include "pch.h"
export module KeyboardTranslator:SequenceDictionary;

import std.core;

export class SequenceDictionary
{
public:
	SequenceDictionary() = default;
	~SequenceDictionary() = default;
	SequenceDictionary(const SequenceDictionary&) = delete;
	SequenceDictionary& operator=(const SequenceDictionary&) = delete;

	struct ValidString : std::wstring { using std::wstring::length; };
	struct Incomplete {};
	struct Invalid {};
	typedef std::variant<ValidString, Incomplete, Invalid> ResultVariant;

	ResultVariant AppendAndSearch(std::wstring sequence);

	void Clear() noexcept;

private:
	std::wstring m_sequence;
};