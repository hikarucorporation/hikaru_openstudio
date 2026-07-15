	pub fn view(&self, window: window::Id) -> Element<'_, Message> {
		if let Some(gui) = self.clap_host.view(window) {
			return gui.map(Message::ClapHost);
		}

		debug_assert_eq!(window, self.main_window_id);

		let transport = self.arrangement_view.arrangement.transport();
		let now_beats = transport.position.to_beat_time(transport);

		// Guardamos el layout base de la app en una variable limpia
		let main_layout = column![
			row![
				pick_list(None::<FileMenu>, FileMenu::VARIANTS, FileMenu::to_string)
					.on_select(Message::from)
					.handle(PICK_LIST_HANDLE)
					.placeholder("File")
					.style(pick_list_with_radius(5))
					.menu_style(menu_style),
				row![
					button(if transport.playing { pause() } else { play() })
						.style(button_with_radius(button::primary, border::left(5)))
						.padding(padding::horizontal(7).vertical(5))
						.on_press(Message::TogglePlayback),
					button(square())
						.style(button_with_radius(button::primary, border::right(5)))
						.padding(padding::horizontal(7).vertical(5))
						.on_press(Message::Stop),
				],
				number_input(
					1..=99,
					transport.numerator.get().into(),
					4,
					|numerator| Message::ChangedNumerator(numerator as u8),
					Message::ChangedNumeratorText,
					5
				),
				row![
					number_input(
						10..=999,
						transport.bpm.get().into(),
						140,
						|bpm| Message::ChangedBpm(bpm as u16),
						Message::ChangedBpmText,
						border::left(5)
					),
					button(metronome())
						.style(button_with_radius(
							if self.state.metronome {
								button::primary
							} else {
								button::secondary
							},
							0
						))
						.padding(padding::all(5).left(4))
						.on_press(Message::ToggleMetronome),
					button(
						mouse_area(container(gavel()).padding(5)).on_press(Message::TappedBpm)
					)
					.style(button_with_radius(button::primary, border::right(5)))
					.padding(0)
					.on_press_with(|| unreachable!()),
				],
				row![
					mouse_area(
						container(
							if self.state.show_seconds {
								text!(
									"{:02}:{:02}:{:02}",
									transport.position.second() / 60,
									transport.position.second() % 60,
									(transport.position.to_float().fract() * 100.0) as u8
								)
							} else {
								text!(
									"{:03}:{:0digits$}",
									now_beats.bar(transport) + 1,
									now_beats.beat_in_bar(transport) + 1,
									digits = transport.numerator.ilog10() as usize + 1,
								)
							}
							.font(Font::MONOSPACE)
						)
						.padding(padding::horizontal(7).vertical(5))
						.style(container_with_radius(weakest_bordered_box, border::left(5)))
					)
					.on_press(Message::ToggleShowSeconds)
					.interaction(Interaction::Pointer),
					button(arrow_big_right())
						.style(button_with_radius(
							if self.state.autoscroll {
								button::primary
							} else {
								button::secondary
							},
							border::right(5)
						))
						.padding(5)
						.on_press(Message::ToggleAutoscroll),
				],
				space::horizontal(),
				row![
					cpu(),
					text!("{:.1}%", self.arrangement_view.arrangement.load() * 100.0)
						.font(Font::MONOSPACE)
				]
				.spacing(5),
				row![
					button(chart_no_axes_gantt())
						.style(button_with_radius(button::primary, border::left(5)))
						.padding(padding::horizontal(7).vertical(5))
						.on_press_maybe(
							(self.arrangement_view.tab() != Tab::Playlist).then_some(
								Message::Arrangement(
									self.project,
									arrangement_view::Message::ChangedTab(Tab::Playlist)
								)
							)
						),
					button(sliders_vertical())
						.style(button_with_radius(button::primary, 0))
						.padding(padding::horizontal(7).vertical(5))
						.on_press_maybe((self.arrangement_view.tab() != Tab::Mixer).then_some(
							Message::Arrangement(
								self.project,
								arrangement_view::Message::ChangedTab(Tab::Mixer)
							)
						)),
					button(keyboard_music())
						.style(button_with_radius(
							if self.arrangement_view.midi_clip().is_some() {
								button::primary
							} else {
								button::secondary
							},
							border::right(5)
						))
						.padding(padding::horizontal(7).vertical(5))
						.on_press_maybe(
							(self.arrangement_view.midi_clip().is_some()
								&& self.arrangement_view.tab() != Tab::PianoRoll)
								.then_some(Message::Arrangement(
									self.project,
									arrangement_view::Message::ChangedTab(Tab::PianoRoll)
								))
						),
				],
			]
			.align_y(Center)
			.spacing(10),
			vertical_split(
				stack![
					self.file_tree.view().map(Message::FileTree),
					self.files_hovered.then(|| center(plus().size(40.0))
						.style(|_| container::background(Color::BLACK.scale_alpha(ALPHA_2_3))))
				],
				self.arrangement_view
					.view(&self.state, &self.plugins)
					.map(|message| Message::Arrangement(self.project, message)),
				self.state.file_tree_split_at,
				Message::OnDrag
			)
			.on_drag_end(Message::OnDragEnd)
			.on_double_click(Message::OnDoubleClick)
			.strategy(Strategy::Start)
			.focus_delay(Duration::ZERO)
			.style(split_style)
		]
		.padding(10)
		.spacing(10);

		// El stack final maneja las capas superiores (modales de carga, errores y settings)
		stack![
			main_layout,
			
			// Capa de deshabilitación de clicks durante cargas del proyecto
			self.arrangement_view
				.loading()
				.then(|| mouse_area(space().width(Fill).height(Fill))
					.interaction(Interaction::Progress)),
			
			// CAPA MODAL DE CONFIGURACIÓN
			self.config_view.as_ref().map(|config_view| {
				opaque(
					mouse_area(
						container(
							center(
								opaque(
									container(config_view.view().map(Message::ConfigView))
										.width(600)
										.padding(20)
										.style(|_| container::background(Color::from_rgb8(30, 30, 35)))
								)
							)
							.width(Fill)
							.height(Fill)
							.style(|_| container::background(Color::BLACK.scale_alpha(ALPHA_2_3)))
						)
						.width(Fill)
						.height(Fill)
					)
					.on_press(Message::CloseConfigView)
				)
			}),

			// Capa de progresos y errores de samples/plugins faltantes
			self.progress.map(|progress| mouse_area(
				container(
					column![
						bottom_center(self.status.as_deref().map(|scanning| {
							container(
								row![
									"scanning",
									container(
										text(scanning)
											.font(Font::MONOSPACE)
											.wrapping(text::Wrapping::None)
											.ellipsis(text::Ellipsis::Middle)
									)
									.padding(padding::horizontal(10).vertical(5))
									.style(container_with_radius(weakest_bordered_box, 5))
								]
								.align_y(Center)
								.spacing(10),
							)
							.padding(10)
							.style(container_with_radius(weak_bordered_box, 5))
						})),
						column![
							progress_bar(0.0..=1.0, progress).style(progress_bar_with_radius(
								if self.missing_plugins.is_empty()
									&& self.missing_samples.is_empty()
								{
									progress_bar::primary
								} else {
									progress_bar::danger
								},
								5
							)),
							scrollable(
								column(
									self.missing_plugins
										.iter()
										.map(|(name, _)| &**name)
										.enumerate()
										.map(|(i, name)| {
											container(
												row![
													"can't find plugin",
													container(
														text(name.to_string_lossy())
															.font(Font::MONOSPACE)
															.wrapping(text::Wrapping::None)
															.ellipsis(text::Ellipsis::Middle)
													)
													.padding(padding::horizontal(10).vertical(5))
													.style(container_with_radius(
														weakest_bordered_box,
														5
													)),
													row![
														button("Ignore")
															.on_press(Message::FindPlugin(
																i,
																Feedback::Ignore
															))
															.style(button_with_radius(
																button::warning,
																border::left(5)
															)),
														button("Cancel")
															.on_press(Message::FindPlugin(
																i,
																Feedback::Cancel
															))
															.style(button_with_radius(
																button::danger,
																border::right(5)
															))
													]
												]
												.align_y(Center)
												.spacing(10),
											)
											.padding(10)
											.style(container_with_radius(weak_bordered_box, 5))
											.into()
										})
										.chain(
											self.missing_samples
												.iter()
												.map(|(name, _)| &**name)
												.enumerate()
												.map(|(i, name)| {
													container(
														row![
															"can't find sample",
															container(
																text(name)
																	.font(Font::MONOSPACE)
																	.wrapping(text::Wrapping::None)
																	.ellipsis(
																		text::Ellipsis::Middle
																	)
															)
															.padding(
																padding::horizontal(10).vertical(5)
															)
															.style(
																container_with_radius(
																	weakest_bordered_box,
																	5
																)
															),
															row![
																button("Pick")
																	.on_press(
																		Message::FindSampleFileDialog(i)
																	)
																	.style(button_with_radius(
																		button::success,
																		border::left(5)
																	)),
																button("Ignore")
																	.on_press(Message::FindSampleFile(
																		i,
																		Feedback::Ignore
																	))
																	.style(button_with_radius(
																		button::warning,
																		0
																	)),
																button("Cancel")
																	.on_press(Message::FindSampleFile(
																		i,
																		Feedback::Cancel
																	))
																	.style(button_with_radius(
																		button::danger,
																		border::right(5)
																	))
															]
														]
														.align_y(Center)
														.spacing(10),
													)
													.padding(10)
													.style(container_with_radius(
														weak_bordered_box,
														5,
													))
													.into()
												})
										),
								)
								.align_x(Center)
								.spacing(10)
							)
							.spacing(10)
						]
						.align_x(Center)
						.spacing(20),
						space::vertical(),
					]
					.align_x(Center)
					.spacing(20)
				)
				.padding(50)
				.style(|_| container::background(Color::BLACK.scale_alpha(ALPHA_2_3))),
			)
			.interaction(Interaction::Progress))
		]
		.into()
	}
