#pragma once

#include "BlankWindow.g.h"
#include "App.xaml.h"

namespace winrt::simbolmudah_ui::implementation
{
    struct BlankWindow : BlankWindowT<BlankWindow>
    {
        explicit BlankWindow(LibSimbolMudah::KeyboardTranslator const& translator);
        BlankWindow(const BlankWindow&) = delete;
        BlankWindow& operator=(const BlankWindow&) = delete;
        ~BlankWindow();

    private:
        fire_and_forget ShowResult(const LibSimbolMudah::KeyboardTranslator& translator, const hstring& message);

        const com_ptr<App> app;
        const LibSimbolMudah::KeyboardTranslator translator;
        event_token showResultsToken;
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct BlankWindow : BlankWindowT<BlankWindow, implementation::BlankWindow>
    {
    };
}
