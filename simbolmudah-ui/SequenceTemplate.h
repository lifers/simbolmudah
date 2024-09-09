#pragma once
#include <winrt/Microsoft.UI.Xaml.h>

struct SequenceTemplate : winrt::implements<SequenceTemplate, winrt::Microsoft::UI::Xaml::IElementFactory>
{
    SequenceTemplate();
    winrt::Microsoft::UI::Xaml::UIElement GetElement(winrt::Microsoft::UI::Xaml::ElementFactoryGetArgs const& args);
    void RecycleElement(winrt::Microsoft::UI::Xaml::ElementFactoryRecycleArgs const& args);

private:
    const winrt::Microsoft::UI::Xaml::ResourceDictionary resourceCache;

    winrt::Microsoft::UI::Xaml::Controls::Border CreateElement(const winrt::hstring& name);
};