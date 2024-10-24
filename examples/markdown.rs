//! Example of a simple UI layout

use bevy::{
    asset::io::{file::FileAssetReader, AssetSource},
    prelude::*,
    ui,
};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{
    CreateChilden, EntityStyleBuilder, InvokeUiTemplate, TextBuilder, UiBuilder, UiTemplate,
};
use bevy_reactor_obsidian::prelude::*;
use bevy_reactor_signals::SignalsPlugin;
use pulldown_cmark::{HeadingLevel, Options, Parser, Tag};

fn style_test(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .position(ui::PositionType::Absolute)
        .padding(3)
        .left(0)
        .right(0)
        .top(0)
        .bottom(0)
        .row_gap(4)
        .background_color(colors::U1);
}

fn main() {
    App::new()
        .register_asset_source(
            "obsidian_ui",
            AssetSource::build().with_reader(|| {
                Box::new(FileAssetReader::new(
                    "crates/bevy_reactor_obsidian/src/assets",
                ))
            }),
        )
        .add_plugins((
            DefaultPlugins,
            SignalsPlugin,
            StyleBuilderPlugin,
            ObsidianUiPlugin,
        ))
        .add_systems(Startup, setup_view_root)
        .add_systems(Update, close_on_esc)
        .run();
}

fn setup_view_root(world: &mut World) {
    let camera = world.spawn((Camera::default(), Camera2d)).id();
    world
        .spawn(Node::default())
        .insert(TargetCamera(camera))
        .styles((typography::text_default, style_test))
        .create_children(|builder| {
            builder.invoke(Markdown {
                text: "
# Introduction
A *Contranym* is a word that can mean the opposite of itself. For example, the word 'cleave' can
mean to cut apart or to stick together.

## Examples
- **Sanction**: to approve or to impose a penalty
- **Dust**: to remove dust or to add dust
- **Left**: to depart or to remain
- **Oversight**: to oversee or to overlook

Most contranyms are context-dependent, and their meanings can change based on how they are used
in a sentence.

The word 'cleave' is an outlier, as it was originally two different words which are homonyms.
Most other contrynms are derived from a single word that has evolved over time.
                "
                .to_string(),
            });
        });
}

pub fn close_on_esc(input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

struct Markdown {
    text: String,
}

impl UiTemplate for Markdown {
    fn build(&self, builder: &mut UiBuilder) {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        let mut parser = Parser::new_ext(&self.text, options);
        let font_handle = builder.world_mut().resource::<AssetServer>().load(
            "embedded://bevy_reactor_obsidian/assets/fonts/Inter/static/Inter_18pt-Medium.ttf",
        );
        let font = TextFont {
            font: font_handle,
            font_size: 14.0,
            ..Default::default()
        };
        process_markdown(builder, &mut parser, &font);
    }
}

fn process_markdown(builder: &mut UiBuilder, parser: &mut Parser, font: &TextFont) {
    while let Some(event) = parser.next() {
        // println!("Event: {:?}", event);
        match event {
            pulldown_cmark::Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    let mut elt = builder.spawn((
                        TextLayout::default(),
                        Text::default(),
                        font.clone(),
                        Node {
                            margin: UiRect::ZERO.with_bottom(ui::Val::Px(8.0)),
                            ..default()
                        },
                    ));
                    elt.create_children(|builder| {
                        process_markdown(builder, parser, font);
                    });
                }

                Tag::Heading {
                    level,
                    id: _id,
                    classes: _classes,
                    attrs: _attrs,
                } => {
                    let font_handle = builder.world_mut().resource::<AssetServer>().load("embedded://bevy_reactor_obsidian/assets/fonts/Inter/static/Inter_18pt-Bold.ttf");
                    let font = TextFont {
                        font: font_handle,
                        font_size: match level {
                            HeadingLevel::H1 => 24.0,
                            HeadingLevel::H2 => 20.0,
                            HeadingLevel::H3 => 18.0,
                            HeadingLevel::H4 => 16.0,
                            _ => 14.0,
                        },
                        ..Default::default()
                    };
                    let mut elt = builder.spawn((
                        TextLayout::default(),
                        Text::default(),
                        font.clone(),
                        Node {
                            margin: UiRect::ZERO.with_bottom(ui::Val::Px(8.0)),
                            ..default()
                        },
                    ));
                    elt.create_children(|builder| {
                        process_markdown(builder, parser, &font);
                    });
                }

                Tag::BlockQuote(_block_quote_kind) => todo!(),
                Tag::CodeBlock(_code_block_kind) => todo!(),
                Tag::HtmlBlock => todo!(),
                Tag::List(_) => {
                    let mut elt = builder.spawn((Node {
                        margin: UiRect::ZERO
                            .with_bottom(ui::Val::Px(8.0))
                            .with_left(ui::Val::Px(16.0)),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },));
                    elt.create_children(|builder| {
                        process_markdown(builder, parser, font);
                    });
                }
                Tag::Item => {
                    let mut elt = builder.spawn((Node {
                        // margin: UiRect::ZERO.with_bottom(ui::Val::Px(0.8)),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },));
                    elt.create_children(|builder| {
                        builder.text("•  ");
                        let mut elt = builder.spawn((
                            TextLayout::default(),
                            Text::default(),
                            font.clone(),
                            Node {
                                margin: UiRect::ZERO.with_bottom(ui::Val::Px(8.0)),
                                ..default()
                            },
                        ));
                        elt.create_children(|builder| {
                            process_markdown(builder, parser, font);
                        });
                    });
                }
                Tag::FootnoteDefinition(_cow_str) => todo!(),
                Tag::DefinitionList => todo!(),
                Tag::DefinitionListTitle => todo!(),
                Tag::DefinitionListDefinition => todo!(),
                Tag::Table(_vec) => todo!(),
                Tag::TableHead => todo!(),
                Tag::TableRow => todo!(),
                Tag::TableCell => todo!(),
                Tag::Emphasis => {
                    let font_handle = builder.world_mut().resource::<AssetServer>().load("embedded://bevy_reactor_obsidian/assets/fonts/Inter/static/Inter_18pt-MediumItalic.ttf");
                    process_markdown(
                        builder,
                        parser,
                        &TextFont {
                            font: font_handle,
                            ..*font
                        },
                    );
                }
                Tag::Strong => {
                    let font_handle = builder.world_mut().resource::<AssetServer>().load("embedded://bevy_reactor_obsidian/assets/fonts/Inter/static/Inter_18pt-Bold.ttf");
                    process_markdown(
                        builder,
                        parser,
                        &TextFont {
                            font: font_handle,
                            ..*font
                        },
                    );
                }
                Tag::Strikethrough => {
                    process_markdown(builder, parser, font);
                }
                Tag::Link {
                    link_type: _link_type,
                    dest_url: _dest_url,
                    title: _title,
                    id: _id,
                } => todo!(),
                Tag::Image {
                    link_type: _link_type,
                    dest_url: _dest_url,
                    title: _title,
                    id: _id,
                } => todo!(),
                Tag::MetadataBlock(_metadata_block_kind) => todo!(),
            },

            pulldown_cmark::Event::End(_tag_end) => {
                // assert_eq!(tag_end, tag.into());
                break;
            }

            pulldown_cmark::Event::Text(cow_str) => {
                builder.spawn((Text(cow_str.into()), font.clone()));
            }

            pulldown_cmark::Event::Code(_cow_str) => {
                // builder.text(cow_str.to_string());
            }

            pulldown_cmark::Event::InlineMath(_cow_str) => {
                // builder.text(cow_str.to_string());
            }

            pulldown_cmark::Event::DisplayMath(_cow_str) => {
                // builder.text(cow_str.to_string());
            }

            pulldown_cmark::Event::Html(_cow_str) => {
                // builder.text(cow_str.to_string());
            }

            pulldown_cmark::Event::InlineHtml(_cow_str) => {
                // builder.text(cow_str.to_string());
            }

            pulldown_cmark::Event::FootnoteReference(_cow_str) => {
                // builder.text(cow_str.to_string());
            }

            pulldown_cmark::Event::SoftBreak => {
                builder.spawn((Text(" ".to_string()), font.clone()));
            }

            pulldown_cmark::Event::HardBreak => {
                builder.spawn((Text("\n".to_string()), font.clone()));
            }

            pulldown_cmark::Event::Rule => {
                // builder.text("----");
            }

            pulldown_cmark::Event::TaskListMarker(_) => {}
        }
    }
}
