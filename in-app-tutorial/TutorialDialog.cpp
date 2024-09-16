module;
#include <corecrt_memcpy_s.h>
module TutorialDialog;

import pcm;
import :WelcomeView;
import :NormalView;
import :UnicodeView;
import :SearchView;
import :ClosingView;

using namespace winrt;
using namespace Microsoft::UI::Xaml;
using namespace Windows::Foundation;
using namespace LibSimbolMudah;
namespace
{
    thread_local Controls::ContentDialog tutorialDialog{ nullptr };
    thread_local bool hookPopupState{ false };

    Controls::Grid CreateTutorialContent(
        ResourceDictionary const& resCache, EventHandler<bool> const& hookPopup)
    {
        using namespace tut;
        const Controls::FlipView flipView{};
        flipView.VerticalAlignment(VerticalAlignment::Top);
        flipView.Items().ReplaceAll({
            WelcomeView(resCache),
            NormalView(resCache, hookPopup, hookPopupState),
            UnicodeView(resCache, hookPopup),
            SearchView(resCache, hookPopup),
            ClosingView(resCache)
        });

        const Controls::PipsPager pipsPager{};
        pipsPager.HorizontalAlignment(HorizontalAlignment::Center);
        pipsPager.NumberOfPages(flipView.Items().Size());
        pipsPager.SelectedPageIndex(flipView.SelectedIndex());
        
        flipView.SelectionChanged([weak{ make_weak(pipsPager) }](IInspectable const& src, Controls::SelectionChangedEventArgs const& args)
        {
            if (const auto pager{ weak.get() }; pager && args.OriginalSource() != pager.as<IInspectable>())
            {
                pager.SelectedPageIndex(src.as<Controls::FlipView>().SelectedIndex());
            }
        });
        pipsPager.SelectedIndexChanged([weak{ make_weak(flipView) }](Controls::PipsPager const& src, auto&&)
        {
            if (const auto flip{ weak.get() }; flip)
            {
                flip.SelectedIndex(src.SelectedPageIndex());
            }
        });

        const Controls::RowDefinition flipRow{};
        flipRow.Height(GridLengthHelper::FromValueAndType(1, GridUnitType::Star));
        const Controls::RowDefinition pipsRow{};

        const Controls::Grid grid{};
        grid.RowDefinitions().ReplaceAll({ flipRow, pipsRow });
        grid.Children().ReplaceAll({ flipView, pipsPager });
        grid.SetRow(flipView, 0);
        grid.SetRow(pipsPager, 1);
        return grid;
    }

    Controls::ContentDialog CreateTutorialDialog(
        ResourceDictionary const& resCache, EventHandler<bool> const& hookPopup)
    {
        const Controls::StackPanel panel{};

        const Controls::ContentDialog dialog{};
        dialog.Title(box_value(L"Tutorial"));
        dialog.Content(CreateTutorialContent(resCache, hookPopup));
        dialog.CloseButtonText(L"Got it!");
        dialog.DefaultButton(Controls::ContentDialogButton::Close);
        return dialog;
    }
}

namespace winrt::in_app_tutorial::implementation
{
    void TutorialDialog::Initialize(ResourceDictionary const& resCache, EventHandler<bool> const& hookPopup)
    {
        tutorialDialog = CreateTutorialDialog(resCache, hookPopup);
    }

    Controls::ContentDialog TutorialDialog::GetDialog() { return tutorialDialog; }
}
