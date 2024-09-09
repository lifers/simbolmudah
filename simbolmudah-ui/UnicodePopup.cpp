#include "pch.hpp"
#include "UnicodePopup.h"
#include "SequenceTemplate.h"

using namespace winrt;

namespace
{
    using namespace Microsoft::UI::Xaml;
    using namespace Windows::Foundation::Collections;

    constinit auto hexChars{ L"0123456789ABCDEFabcdef" };

    constexpr bool IsHexChar(wchar_t c)
    {
        for (uint8_t i{ 0 }; i < 22; ++i)
        {
            if (c == hexChars[i]) { return true; }
        }
        return false;
    }

    Controls::TextBlock CreateResultText()
    {
        const auto& res{ Application::Current().Resources() };
        WINRT_ASSERT(res.HasKey(box_value(L"DisplayTextBlockStyle")));
        const auto& labelStyle{ res.Lookup(box_value(L"DisplayTextBlockStyle")).as<Style>() };

        const Controls::TextBlock label{};
        label.Style(labelStyle);
        label.FontSize(16);
        label.HorizontalAlignment(HorizontalAlignment::Center);
        label.VerticalAlignment(VerticalAlignment::Center);
        return label;
    }

    Controls::TextBlock CreateDescText()
    {
        const auto& res{ Application::Current().Resources() };
        WINRT_ASSERT(res.HasKey(box_value(L"TextFillColorSecondaryBrush")));
        const auto& descStyle{ res.Lookup(box_value(L"TextFillColorSecondaryBrush")).as<Media::Brush>() };

        Controls::TextBlock desc{};
        desc.Foreground(descStyle);
        desc.TextWrapping(TextWrapping::Wrap);
        return desc;
    }

    Controls::Grid CreateResultGrid(Controls::TextBlock const& result, Controls::TextBlock const& desc)
    {
        const auto& res{ Application::Current().Resources() };
        WINRT_ASSERT(res.HasKey(box_value(L"LayerOnAcrylicFillColorDefaultBrush")));
        const auto& boxStyle{ res.Lookup(box_value(L"LayerOnAcrylicFillColorDefaultBrush")).as<Media::Brush>() };

        const Controls::Border box{};
        box.Background(boxStyle);
        box.VerticalAlignment(VerticalAlignment::Center);
        box.HorizontalAlignment(HorizontalAlignment::Center);
        box.Width(32);
        box.Height(32);
        box.CornerRadius(CornerRadiusHelper::FromUniformRadius(4));
        box.Child(result);

        const Controls::ColumnDefinition boxCol{};
        boxCol.Width(GridLengthHelper::FromPixels(32));
        const Controls::ColumnDefinition descCol{};

        const Controls::Grid grid{};
        grid.VerticalAlignment(VerticalAlignment::Center);
        grid.Shadow(Media::ThemeShadow());
        grid.ColumnDefinitions().ReplaceAll({ boxCol, descCol });
        grid.ColumnSpacing(8);
        grid.Children().ReplaceAll({ box, desc });
        grid.SetColumn(box, 0);
        grid.SetColumn(desc, 1);
        return grid;
    }

    Controls::ItemsRepeater CreateSequenceRepeater(IObservableVector<hstring> const& src)
    {
        const Controls::StackLayout layout{};
        layout.Orientation(Controls::Orientation::Horizontal);
        layout.Spacing(2);

        const Controls::ItemsRepeater repeater{};
        repeater.ItemsSource(src);
        repeater.ItemTemplate(make<SequenceTemplate>());
        repeater.VerticalAlignment(VerticalAlignment::Center);
        repeater.Layout(layout);
        return repeater;
    }

    Controls::Border CreateTopBox(Controls::ItemsRepeater const& repeater)
    {
        const auto& res{ Application::Current().Resources() };
        WINRT_ASSERT(res.HasKey(box_value(L"LayerOnAcrylicFillColorDefaultBrush")));
        WINRT_ASSERT(res.HasKey(box_value(L"ControlElevationBorderBrush")));
        const auto& boxBackground{ res.Lookup(box_value(L"LayerOnAcrylicFillColorDefaultBrush")).as<Media::Brush>() };
        const auto& boxBorder{ res.Lookup(box_value(L"ControlElevationBorderBrush")).as<Media::Brush>() };

        const Controls::Border box{};
        box.Background(boxBackground);
        box.BorderBrush(boxBorder);
        box.BorderThickness(ThicknessHelper::FromUniformLength(1));
        box.CornerRadius(CornerRadiusHelper::FromUniformRadius(4));
        box.Height(40);
        box.Child(repeater);
        return box;
    }

    Controls::Page CreateInnerPage(Controls::Border const& topBox, Controls::Grid const& bottomGrid)
    {
        const Controls::StackPanel panel{};
        panel.Padding(ThicknessHelper::FromUniformLength(8));
        panel.Spacing(8);
        panel.Children().ReplaceAll({ topBox, bottomGrid });

        const Controls::Page page{};
        page.Content(panel);
        return page;
    }
}

namespace winrt::simbolmudah_ui
{
    using namespace LibSimbolMudah;

    UnicodePopup::UnicodePopup(SequenceDefinition const& seqdef) :
        seqdef(seqdef), hexCodes(single_threaded_observable_vector(std::vector<hstring>())),
        resultText(CreateResultText()), descText(CreateDescText()),
        innerPage(CreateInnerPage(
            CreateTopBox(CreateSequenceRepeater(this->hexCodes)),
            CreateResultGrid(this->resultText, this->descText)))
    {
        this->ResetState();
    }

    void UnicodePopup::ShowAnswer() const
    {
        std::wstring numstring{};
        for (const auto& letter: this->hexCodes)
        {
            if (!IsHexChar(letter.c_str()[0]))
            {
                this->resultText.Text(L"");
                this->descText.Text(L"Invalid hexadecimal number. Press enter to cancel.");
                return;
            }
            numstring.append(letter.c_str());
        }

        SequenceDescription desc{};
        try
        {
            const auto num{ std::stoul(numstring, nullptr, 16) };
            if (num > 0x10FFFF)
            {
                this->resultText.Text(L"");
                this->descText.Text(L"Number out of range. Press enter to cancel.");
                return;
            }

            desc = this->seqdef.GetLocalizedName(num);
        }
        catch (const std::invalid_argument&)
        {
            this->resultText.Text(L"");
            this->descText.Text(L"Invalid hexadecimal number. Press enter to cancel.");
            return;
        }
        catch (const std::out_of_range&)
        {
            this->resultText.Text(L"");
            this->descText.Text(L"Number out of range. Press enter to cancel.");
            return;
        }
        catch (winrt::hresult_error const& e)
        {
            this->resultText.Text(L"");
            this->descText.Text(e.message());
            return;
        }

        this->resultText.Text(desc.result);
        this->descText.Text(desc.description);
    }

    void UnicodePopup::ResetState() const
    {
        this->hexCodes.Clear();
        this->resultText.Text(L"");
        this->descText.Text(L"Enter a hexadecimal number.");
    }
}
