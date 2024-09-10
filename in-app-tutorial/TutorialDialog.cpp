module;
#define WIN32_LEAN_AND_MEAN
#include <Windows.h>
#include <crtdbg.h>
module TutorialDialog;

using namespace winrt;
using namespace Microsoft::UI::Xaml;

import :WelcomeView;
import :NormalView;
import :UnicodeView;
import :SearchView;
import :ClosingView;

namespace
{
    thread_local Controls::ContentDialog tutorialDialog{ nullptr };

    Controls::FlipView CreateTutorialFlipView(ResourceDictionary const& resCache)
    {
        using namespace tut;
        const Controls::FlipView flipView{};
        flipView.VerticalAlignment(VerticalAlignment::Center);
        flipView.Items().ReplaceAll({
            WelcomeView(resCache),
            NormalView(resCache),
            UnicodeView(resCache),
            SearchView(resCache),
            ClosingView(resCache)
        });
        return flipView;
    }

    Controls::ContentDialog CreateTutorialDialog(ResourceDictionary const& resCache)
    {
        const Controls::ContentDialog dialog{};
        dialog.Title(box_value(L"Tutorial"));
        dialog.Content(CreateTutorialFlipView(resCache));
        dialog.CloseButtonText(L"Got it!");
        dialog.DefaultButton(Controls::ContentDialogButton::Close);
        return dialog;
    }
}

namespace winrt::in_app_tutorial::implementation
{
    void TutorialDialog::Initialize(ResourceDictionary const& resCache)
    {
        tutorialDialog = CreateTutorialDialog(resCache);
    }

    Controls::ContentDialog TutorialDialog::AttachTutorialDialog(XamlRoot const& xamlRoot)
    {
        tutorialDialog.XamlRoot(xamlRoot);
        return tutorialDialog;
    }
}
