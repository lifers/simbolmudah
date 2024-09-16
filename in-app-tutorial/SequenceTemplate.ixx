module;
#include <corecrt_memcpy_s.h>
export module Common:SequenceTemplate;

import pcm;

namespace com {
    using namespace winrt;
    using namespace Microsoft::UI::Xaml;
    export struct SequenceTemplate : implements<SequenceTemplate, IElementFactory>
    {
        SequenceTemplate();
        UIElement GetElement(ElementFactoryGetArgs const& args);
        void RecycleElement(ElementFactoryRecycleArgs const& args);

    private:
        const ResourceDictionary resourceCache;
    };

    export Controls::Border CreateElement(ResourceDictionary const& resCache, hstring const& name);
}