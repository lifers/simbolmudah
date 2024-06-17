#include "pch.h"
export module ResultSender;

import std.core;

export namespace ResultSender
{
	winrt::fire_and_forget Send(std::span<INPUT> buffer);
	winrt::fire_and_forget TranslateAndSend(std::wstring str);
}