extern crate printpdf;
use printpdf::path::{PaintMode, WindingOrder};
use printpdf::*;

use std::convert::From;
use std::fs::File;
use std::io::BufWriter;
// use std::string;
// use std::mem::transmute;

use chrono::prelude::*;
use ttf_parser::Face;

pub struct IOrder {
    pub comentario: Option<String>,
    pub items: Vec<Item>,
    pub dscto_cupon_gasto_envio: f32,
    pub dscto_cupon_subtotal: f32,
    pub dscto_puntos: f32,
    pub cupones: Option<Vec<Cupon>>,
    pub fechas: Fechas,
    pub tipo_entrega: TipoEntrega,
    pub drop_off: Option<DropOff>,
    pub courier: Courier,
    pub cliente: Cliente,
    pub sucursal: Option<Sucursal>,
    pub plataforma: Plataforma,
    pub correlativo: i64,
    pub codigo: String,
    pub comercio: Comercio,
    pub pago: Pago,
    pub sub_total: i32,
    pub gastos_envio: i32,
}

/// Representa un producto o ítem dentro de la orden.
pub struct Item {
    pub cantidad: f32, // Equivalente a `number` en TS
    pub nombre: String,
    pub precio: f32,
    pub opciones: Option<Vec<IOpciones>>,
    pub comentario: Option<String>,
}

/// Representa una opción o modificador de un ítem (equivalente a IOpciones).
pub struct IOpciones {
    pub modificador: String,
    pub cantidad: i32,
    pub opcion: String,
}

/// Representa un cupón aplicado a la orden.
pub struct Cupon {
    pub codigo: String,
}

/// Fechas relevantes para la orden, como hora de pago, hora de salida de cocina, etc.
pub struct Fechas {
    pub fecha_salida_cocina_estimada: String,
    pub fecha_entrega_min: String,
    pub tz: String,
    pub fecha_pago: String,
}

/// Tipo de entrega (por ejemplo, 1 = Delivery, 2 = Retiro en local).
pub struct TipoEntrega {
    pub id: i32,
}

/// Información opcional cuando es envío a domicilio.
pub struct DropOff {
    pub tipo_entrega: Option<String>,
    pub direccion: Option<String>,
}

/// Información sobre el courier (por ejemplo, IdCourier = -2 cuando es propio).
pub struct Courier {
    pub id_courier: i32,
}

/// Datos del cliente.
pub struct Cliente {
    pub telefono: Option<String>,
    pub nombre: Option<String>,
    pub nro: Option<String>,
}

/// Sucursal (en caso de retiro en tienda).
pub struct Sucursal {
    pub nombre: Option<String>,
}

/// Plataforma desde donde llega la orden (AGIL, etc.).
pub struct Plataforma {
    pub codigo: Option<String>,
    pub nombre: Option<String>,
}

/// Datos del comercio o restaurante que recibe la orden.
pub struct Comercio {
    pub nombre: Option<String>,
}

/// Información del pago (medio de pago, etc.).
pub struct Pago {
    pub medio_pago: MedioPago,
}

/// Medio de pago específico (efectivo, tarjeta, etc.).
pub struct MedioPago {
    pub nombre: Option<String>,
}

struct PdfResources<'a> {
    font: IndirectFontRef,
    face: Face<'a>,
    upem: f32,
    font_light: IndirectFontRef,
    face_light: Face<'a>,
    upem_light: f32,
    page_width: f32,
    page_height: f32,
    doc: PdfDocumentReference,
    page: PdfPageIndex,
    layer: PdfLayerIndex,
}

impl<'a> PdfResources<'a> {
    fn new(altura_extra: f32) -> Self {
        let page_width = 80.0;
        let page_height = 169.0 + altura_extra;
        let (doc, page, layer) =
            PdfDocument::new("Ticket", Mm(page_width as f32), Mm(page_height as f32), "");

        let font_data: &[u8] = include_bytes!("../assets/fonts/segoe-ui-bold.ttf") as &[u8];
        let face: Face<'_> = Face::parse(font_data, 0).expect("No se pudo cargar la fuente");
        let upem: f32 = face.units_per_em() as f32;
        let font: IndirectFontRef = doc.add_external_font(font_data).expect(
            "Failed to add external font. Ensure the font data is correct and file path is valid.",
        );

        let font_ligth_data: &[u8] = include_bytes!("../assets/fonts/segoe-ui.ttf") as &[u8];
        let face_light: Face<'_> =
            Face::parse(font_ligth_data, 0).expect("No se pudo cargar la fuente");
        let upem_light: f32 = face_light.units_per_em() as f32;
        let font_light: IndirectFontRef = doc.add_external_font(font_ligth_data).expect(
            "Failed to add external font. Ensure the font data is correct and file path is valid.",
        );

        PdfResources {
            font,
            face,
            upem,
            face_light,
            upem_light,
            font_light,
            page_width,
            page_height,
            doc,
            page,
            layer,
        }
    }
}

fn pdf(orden: &IOrder) {
    let mut altura_extra: f32  = 0.0;
    let descuento_monto = 0;
    
    if orden.courier.id_courier == -2 {
        altura_extra += 7.0;
    }

    if descuento_monto < 0 {
        altura_extra += 8.0;
    }
    for item in &orden.items {
        altura_extra += 6.0;
        for _i in item.opciones.as_ref().unwrap() {
            altura_extra += 10.0;
        }
    }
    

    let resources = PdfResources::new(altura_extra);

    let current_layer: PdfLayerReference = resources
        .doc
        .get_page(resources.page)
        .get_layer(resources.layer);

    let mut gastos_envio: i32 = 0;
    // let mut extra = 0.0;
    /*
    // CUERPO 0: header
    fn header(
        current_layer: &PdfLayerReference,
        resources: &PdfResources<'_>,
        orden: &IOrder,
    ) -> f32 {
        let mut alto = 0.0;
        // comercio nombre
        set_texto(
            current_layer,
            24.0,
            orden.comercio.nombre.as_ref().unwrap(),
            14.0,
            resources,
            0,
            false,
        );

        // plataforma nombre
        set_texto(
            current_layer,
            16.0,
            orden.plataforma.nombre.as_ref().unwrap(),
            19.0,
            resources,
            0,
            false,
        );

        set_linea_horizontal(current_layer, 52.0, resources);

        // Cliente nombre
        set_texto(
            current_layer,
            24.0,
            orden.cliente.nombre.as_ref().unwrap(),
            60.0,
            resources,
            0,
            false,
        );

        // Cliente Numero
        if orden.courier.id_courier == -2 {
            set_texto(
                current_layer,
                20.0,
                orden.cliente.telefono.as_ref().unwrap(),
                68.0,
                resources,
                0,
                false,
            );

            // nuestro
            let nuestro_string = String::from("*NUESTRO*");
            let nuestro: &String = &nuestro_string;
            
            set_texto(current_layer, 12.0, nuestro, 45.0, resources, -1, false);
            alto += 8.0;
        }

        // CODIGO pedido
        let codigo_pedido_string = &orden.codigo;
        let codigo_pedido: &String = &codigo_pedido_string;
        set_texto(current_layer, 14.0, codigo_pedido, 38.0, resources, 1, true);

        // salida_cocina
        let salida_cocina_string = String::from("Salida Cocina");
        let salida_cocina: &String = &salida_cocina_string;
        set_texto(
            current_layer,
            12.0,
            salida_cocina,
            50.0,
            resources,
            -1,
            true,
        );

        // hora salida cociona
        let salida_cocina = format_datetime(orden.fechas.fecha_salida_cocina_estimada.as_ref());
        let hora_salida_cocina: &String = &salida_cocina.1;
        set_texto(
            current_layer,
            32.0,
            hora_salida_cocina,
            48.0,
            resources,
            1,
            false,
        );
        if orden.tipo_entrega.id == 1 {
            let _ = set_img(current_layer, resources, 5.0, 30.0, 10.0, 10.0, "moto");
        } else {
            let _ = set_img(current_layer, resources, 5.0, 30.0, 10.0, 10.0, "camino");
        }

        // reimpreso

        let copia_string = String::from("*REIMPRESO*"); // TODO
        let copia: &String = &copia_string;
        set_texto(current_layer, 12.0, copia, 5.0, resources, -1, false);
        alto
    }

    fn envio(
        current_layer: &PdfLayerReference,
        resources: &PdfResources<'_>,
        orden: &IOrder,
        extra: f32,
    ) -> f32 {
        // CUERPO 1: Envio

        set_linea_horizontal(current_layer, 62.75 + extra, resources);
        set_linea_horizontal(current_layer, 63.50 + extra, resources);

        // ubicacion
        let ubicacion: String = orden
            .drop_off
            .as_ref()
            .unwrap()
            .direccion
            .clone() // Asegura que dirección es una opción de copia
            .unwrap_or("".to_string());

        set_parrafo(
            current_layer,
            14.0,
            &ubicacion,
            69.0 + extra,
            resources,
            0,
            false,
        );
        // icono ubicacion
        let _ = set_img(
            current_layer,
            resources,
            38.0,
            66.0 + extra as f32,
            4.0,
            5.0,
            "ubicacion",
        );

        // indicacion
        let tipo: String = orden
            .drop_off
            .as_ref()
            .unwrap()
            .tipo_entrega
            .clone()
            .unwrap_or("".to_string());

        set_texto(current_layer, 14.0, &tipo, 74.0 + extra, resources, 0, true);

        // hora pago
        set_texto(
            current_layer,
            14.0,
            &String::from("Hora de Pago"),
            82.0 + extra,
            resources,
            -1,
            true,
        );

        let fecha_pago = format_datetime(&orden.fechas.fecha_pago.as_ref());
        let hora_pago = format!("{}. - {}", fecha_pago.0, fecha_pago.1);
        set_texto(
            current_layer,
            12.0,
            &hora_pago,
            82.0 + extra,
            resources,
            1,
            true,
        );

        // hora entrega
        set_texto(
            current_layer,
            14.0,
            &String::from("Hora de Entrega"),
            87.0 + extra,
            resources,
            -1,
            true,
        );

        let fecha_entrega = format_datetime(&orden.fechas.fecha_entrega_min.as_ref());
        let hora_entrega = format!("{}. - {}", fecha_entrega.0, fecha_entrega.1);
        set_texto(
            current_layer,
            12.0,
            &hora_entrega,
            87.0 + extra,
            resources,
            1,
            true,
        );

        //   altura  = 95.0,
        /*
         Calculo relativo al inicio pero de tamaño variable!!!
        */
        // Comentario cliente
        set_linea_horizontal(&current_layer, 89.0 + extra, &resources);
        set_texto(
            &current_layer,
            12.0,
            &String::from(" Comentario del Cliente: "),
            94.0 + extra,
            &resources,
            -1,
            false,
        );

        let comentario: &String = orden.comentario.as_ref().unwrap();

        let comenetario_co = set_parrafo(
            &current_layer,
            10.0,
            &comentario,
            99.0 + extra,
            &resources,
            -1,
            true,
        );
        resources.page_height as f32 - comenetario_co.1 + extra
    }

    extra += header(&current_layer, &resources, orden);
    let mut altura_nueva = envio(&current_layer, &resources, orden, extra);

    set_lineas_verticales(&current_layer, 89.0 + extra, altura_nueva, &resources);
    set_linea_horizontal(&current_layer, altura_nueva, &resources);

    // Cuerpo 2: pedidos
    // cubiertos
    set_linea_horizontal(&current_layer, altura_nueva + 5.0, &resources);
    set_linea_horizontal(&current_layer, altura_nueva + 5.75, &resources);

    altura_nueva = altura_nueva + 8.0;
    let _ = set_img(
        &current_layer,
        &resources,
        37.0,
        altura_nueva as f32,
        6.0,
        6.0,
        "cubiertos",
    );
    // productos
    // modificadores
    for item in &orden.items {
        altura_nueva = altura_nueva + 5.0;
        let producto = item.cantidad.to_string() + " X " + &item.nombre;
        set_texto(
            &current_layer,
            13.0,
            &producto,
            altura_nueva,
            &resources,
            -1,
            false,
        );
        let precio = format_clp((item.precio * item.cantidad) as i32);
        set_texto(
            &current_layer,
            13.0,
            &precio,
            altura_nueva,
            &resources,
            1,
            false,
        );
        // let ubicacion: String = orden
        //     .drop_off
        //     .as_ref()
        //     .unwrap()
        //     .direccion
        //     .clone() // Asegura que dirección es una opción de copia
        //     .unwrap_or("".to_string());

        // altura_nueva = altura_nueva + 2.0;
        for modi in item.opciones.as_ref().unwrap() {
            altura_nueva = altura_nueva + 5.0;
            let mut text_modi = String::new();
            text_modi.push_str("   - ");
            text_modi.push_str(&modi.modificador);
            set_texto(
                &current_layer,
                13.0,
                &text_modi,
                altura_nueva,
                &resources,
                -1,
                true,
            );
            altura_nueva = altura_nueva + 5.0;
            text_modi = String::new();
            text_modi.push_str("     ");
            text_modi.push_str(&modi.cantidad.to_string());
            text_modi.push_str(" X   ");
            text_modi.push_str(&modi.opcion);

            set_texto(
                &current_layer,
                13.0,
                &text_modi,
                altura_nueva,
                &resources,
                -1,
                true,
            );
        }

        // set_linea_horizontal(&current_layer, altura_nueva, &resources); //todo borrar despues
        // println!("{}", altura_nueva);
    }

    // FOOTER: pagos
    /*
     Calculo relativo al final
    */

    // descuentos
    // - 1 puntos
    // - 2
    // - 3
    let descuento_nombre = "PUNTOS?".to_string();
    // recorrer el objeto, si uno de los 3 descuentos tiene valor ese sera el que se muestra
    // ya que solo puede usarce un valor por logica de negocio

    // Costo despacho
    if descuento_monto < 0 {
        extra_neg = -7.0;
        let str_descuento_monto = format_clp(descuento_monto);
        set_texto(
            &current_layer,
            16.0,
            &descuento_nombre,
            -37.0,
            &resources,
            -1,
            true,
        );
        set_texto(
            &current_layer,
            16.0,
            &str_descuento_monto,
            -37.0,
            &resources,
            1,
            false,
        );
    }
    */
    // Cost subtotal
    set_texto(
        &current_layer,
        16.0,
        &String::from("Subtotal"),
        44.0,
        &resources,
        -1,
        true,
    );
    let costo_total = format_clp(orden.sub_total as i32);
    set_texto(
        &current_layer,
        16.0,
        &costo_total,
        44.0,
        &resources,
        1,
        false,
    );
    set_separacion(&resources, &current_layer, 50.0, "dinero");
    if orden.tipo_entrega.id == 1 {
        gastos_envio = orden.gastos_envio;
        let str_gastos_envio = format_clp(gastos_envio);
        set_texto(
            &current_layer,
            16.0,
            &String::from("Despacho"),
            37.0,
            &resources,
            -1,
            true,
        );
        set_texto(
            &current_layer,
            16.0,
            &str_gastos_envio,
            37.0,
            &resources,
            1,
            false,
        );
    }
    // costo total
    set_texto(
        &current_layer,
        16.0,
        &String::from("Total"),
        30.0,
        &resources,
        -1,
        true,
    );
    let costo_total = format_clp(orden.sub_total + gastos_envio + descuento_monto);
    set_texto(
        &current_layer,
        16.0,
        &costo_total,
        30.0,
        &resources,
        1,
        false,
    );

    // no incluye ...
    let disclaimer = String::from("* Total no incluye propina ni cuota de servicio.");
    set_texto(
        &current_layer,
        9.0,
        &disclaimer,
        24.0,
        &resources,
        -1,
        true,
    );

    // medio de pago ...
    let medio_pago = orden.pago.medio_pago.nombre.as_ref().unwrap();
    set_texto(
        &current_layer,
        20.0,
        medio_pago,
        17.0,
        &resources,
        1,
        false,
    );

    // power by agil
    let power_agil = String::from("powered by Agil");
    set_texto(
        &current_layer,
        12.0,
        &power_agil,
        10.0,
        &resources,
        0,
        true,
    );

    // FIN !
    let mut buffer: BufWriter<File> = BufWriter::new(File::create("test_working.pdf").unwrap());
    resources.doc.save(&mut buffer).unwrap();
}

fn set_separacion(resources: &PdfResources<'_>, current_layer: &PdfLayerReference, y: f32, icono:&str ) {
    set_linea_horizontal(current_layer, y + 1.5, resources);
    set_linea_horizontal(current_layer, y + 2.75, resources);
    let _ = set_img(
        current_layer,
        resources,
        37.0,
        y - 0.75,
        6.0,
        6.0,
        icono,
    );
}

fn format_datetime(iso_date: &str) -> (String, String) {
    // Parseamos la fecha ISO
    let datetime = DateTime::parse_from_rfc3339(iso_date).expect("Error parsing ISO date");

    // Formateamos la fecha como "DD MMM"
    let date = datetime.format("%d %b").to_string();

    // Formateamos la hora como "HH:MM"
    let time = datetime.format("%H:%M").to_string();

    (date, time)
}

fn format_clp(amount: i32) -> String {
    let mut formatted = String::new();
    let amount_str = amount.to_string();
    let mut count = 0;

    for ch in amount_str.chars().rev() {
        if count > 0 && count % 3 == 0 {
            formatted.push('.');
        }
        formatted.push(ch);
        count += 1;
    }

    formatted = formatted.chars().rev().collect();
    format!("${}", formatted)
}

fn set_linea_horizontal(current_layer: &PdfLayerReference, altura: f32, resources: &PdfResources) {
    // let i:f32  = (resources.page_height - altura) as f32;
    let mut i = altura;
    let y = i as f32;
    let l = 5.0;
    let r = 75.0;
    let points = vec![
        (Point::new(Mm(l), Mm(y)), false),
        (Point::new(Mm(l), Mm(y)), false),
        (Point::new(Mm(r), Mm(y)), false),
        (Point::new(Mm(r), Mm(y)), false),
    ];

    let line = Polygon {
        rings: vec![points],
        mode: PaintMode::FillStroke,
        winding_order: WindingOrder::NonZero,
    };
    current_layer.add_polygon(line);
}

fn set_lineas_verticales(
    current_layer: &PdfLayerReference,
    inicio: f32,
    fin: f32,
    resources: &PdfResources,
) {
    let y_inicio = (resources.page_height as f32 - inicio) as f32;
    let y_fin = (resources.page_height as f32 - fin) as f32;
    let l = 5.0;
    let r = 75.0;

    let points_izq = vec![
        (Point::new(Mm(l), Mm(y_inicio)), false),
        (Point::new(Mm(l), Mm(y_fin)), false),
    ];

    let line_izq = Line {
        points: points_izq,
        is_closed: false,
    };

    current_layer.add_line(line_izq);

    let points_der = vec![
        (Point::new(Mm(r), Mm(y_inicio)), false),
        (Point::new(Mm(r), Mm(y_fin)), false),
    ];

    let line_der = Line {
        points: points_der,
        is_closed: false,
    };

    current_layer.add_line(line_der);
}

fn set_img(
    current_layer: &PdfLayerReference,
    resources: &PdfResources,
    x: f32,
    y: f32,
    mm_x: f32,
    mm_y: f32,
    icono: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    const DPI: f32 = 300.0;

    // fn mm_to_px(mm: f32) -> usize {
    //     // There are 25.4 millimeters in an inch.
    //     let pixels = (mm / 25.4) * DPI;
    //     // Convert inches to pixels using the specified DPI
    //     pixels.round() as usize
    // }
    fn px_to_mm(px: f32) -> f32 {
        // There are 25.4 millimeters in an inch.
        let mm = (px / DPI) * 25.4;
        mm as f32
    }

    let path_icono = "assets/img/".to_owned() + icono + ".bmp";

    let mut image_file = File::open(path_icono).unwrap();
    let img: Image =
        Image::try_from(image_crate::codecs::bmp::BmpDecoder::new(&mut image_file).unwrap())
            .unwrap();

    let base_scale_x = px_to_mm(img.image.width.0 as f32);
    let base_scale_y = px_to_mm(img.image.height.0 as f32);

    // println!("{}[px] X {}[px]", img.image.width.0, img.image.height.0);
    // println!(" {}[mm] x {}[mm]", base_scale_x, base_scale_y,);
    // scale_x * base_scale_x = mm_x
    let scale_x = mm_x / base_scale_x;
    let scale_y = mm_y / base_scale_y;

    let image_transform: ImageTransform = ImageTransform {
        translate_x: Some(Mm(x)),
        translate_y: Some(Mm(y)),
        rotate: None,
        scale_x: Some(scale_x),
        scale_y: Some(scale_y),
        dpi: Some(DPI), // Usar los nuevos DPI aquí también
    };

    img.add_to_layer(current_layer.clone(), image_transform);

    Ok(())
}

/// Dibuja texto en forma de párrafo, con máximo 70mm de ancho por línea.
fn set_parrafo(
    current_layer: &PdfLayerReference,
    font_size: f32,
    text: &str,
    altura: f32,
    resources: &PdfResources,
    tipo: i8,
    light: bool,
) -> (f32, f32) {
    // 1. Elegir el font correcto (regular o light)
    let (font_use, face_use, upem_use) = if light {
        (
            &resources.font_light,
            &resources.face_light,
            resources.upem_light,
        )
    } else {
        (&resources.font, &resources.face, resources.upem)
    };

    // 2. Calcular factor de escala de la fuente
    let scale_factor = font_size / upem_use;

    // 3. Máximo ancho en mm permitido para cada línea
    let max_width_mm = 60.0;

    // 4. Dividir el texto en palabras
    //    * Podrías hacer un split más robusto si necesitas separar
    //      saltos de línea, tabuladores, etc.
    let words: Vec<&str> = text.split_whitespace().collect();

    // 5. Función auxiliar para medir el ancho de una palabra
    //    en milímetros usando la `face_use`.
    let measure_word_mm = |w: &str| -> f32 {
        let width_points: f32 = w
            .chars()
            .filter_map(|c| face_use.glyph_index(c))
            .map(|glyph_id| face_use.glyph_hor_advance(glyph_id).unwrap_or(0) as f32)
            .sum::<f32>()
            * scale_factor;

        // Convertir de puntos tipográficos a mm
        width_points * 0.352778
    };

    // 6. Construir “líneas” respetando el max_width_mm
    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();
    let mut current_line_width: f32 = 0.0;

    let space_width_mm = measure_word_mm(" "); // ancho de un espacio en mm

    for (i, word) in words.iter().enumerate() {
        // Medir la palabra
        let word_width = measure_word_mm(word);
        let sep = if i == 0 || current_line.is_empty() {
            "" // si es la primera palabra de la línea, no ponemos espacio
        } else {
            " "
        };
        let extra_width = if sep.is_empty() { 0.0 } else { space_width_mm };

        // Revisar si cabe la palabra en la línea actual
        if current_line_width + word_width + extra_width <= max_width_mm {
            // Cabe en la línea
            if current_line.is_empty() {
                current_line = word.to_string();
            } else {
                current_line.push_str(sep);
                current_line.push_str(word);
            }
            // Sumar ancho
            current_line_width += word_width + extra_width;
        } else {
            // No cabe en la línea; guardar la línea actual en `lines`,
            // empezar una nueva con la palabra que no cupo
            lines.push(current_line);
            current_line = word.to_string();
            current_line_width = word_width;
        }
    }

    // Agregar la última línea si quedó algo
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // 7. Ahora dibujamos línea por línea.
    //
    //    - Ajustamos la posición X en base a `tipo`:
    //        tipo < 0 -> alineación izquierda
    //        tipo = 0 -> alineación centrada
    //        tipo > 0 -> alineación derecha
    //
    //    - El “interlineado” o salto de línea suele ser
    //      algo como 1.2 * font_size, o un valor fijo.
    let line_spacing: f32 = font_size * 0.4; // Ajusta a tu gusto
    let mut y_position: f32 = altura;


    // Vamos a devolver la última posición X e Y dibujada.
    // El “último X” no es tan relevante porque cada línea
    // puede tener un X distinto. Si quieres solo dejarlo
    // en 10.0 o en el calculado por la última línea,
    // eso depende de tu uso.
    let mut last_x_position: f32 = 10.0;
    // let mut last_altura = altura;

    for line in &lines {
        // Medir el ancho de la línea para alinear:
        let line_width_mm = measure_word_mm(line);

        let x_position = if tipo < 0 {
            // Izquierda
            10.0
        } else if tipo == 0 {
            // Centrado
            (resources.page_width as f32 - line_width_mm) / 2.0
        } else {
            // Derecha (asumimos que 70mm es el contenedor)
            60.0 - line_width_mm
        };

        // Dibuja la línea
        current_layer.use_text(
            line, // el texto de la línea
            font_size as f32,
            Mm(x_position as f32),
            Mm(y_position as f32),
            font_use,
        );

        last_x_position = x_position;

        // Bajar la posición Y para la siguiente línea
        // OJO: en tu caso usabas “incrementar tamaño en 7.5”
        //      quizá lo quieras fijo
        y_position -= line_spacing;
        // last_altura += line_spacing;
        // println!("{}, {}", last_altura, altura);
    }

    // Devuelve la última posición usada (aprox.)

    (last_x_position, y_position)
}

fn set_texto(
    current_layer: &PdfLayerReference,
    font_size: f32,
    text: &String,
    altura: f32,
    resources: &PdfResources,
    tipo: i8,
    light: bool,
) -> f32 {
    let mut font_use = resources.font.clone();
    let mut face_use = resources.face.clone();
    let mut upem_use = resources.upem.clone();
    if light {
        font_use = resources.font_light.clone();
        face_use = resources.face_light.clone();
        upem_use = resources.upem_light.clone();
    }

    let scale_factor = font_size / upem_use;

    let text_width_points: f32 = text
        .chars()
        .filter_map(|c| face_use.glyph_index(c))
        .map(|glyph_id| face_use.glyph_hor_advance(glyph_id).unwrap_or(0) as f32)
        .sum::<f32>()
        * scale_factor;

    let text_width_mm: f32 = text_width_points * 0.352778;

    let mut x_position: f32 = 5.0; // left
    if tipo == 0 {
        x_position = (resources.page_width as f32 - text_width_mm) / 2.0;
    } else if tipo > 0 {
        // right
        x_position = 75.0 - text_width_mm;
    }


    current_layer.use_text(
        text,
        font_size as f32,
        Mm(x_position as f32),
        Mm(altura as f32),
        &font_use,
    );

    x_position
}

fn main() {
    let orden_ejemplo = IOrder {
        comentario: Some("Orden de ejemplo ".to_string()),
        items: vec![
            Item {
                cantidad: 2.0,
                nombre: "Pizza Napolitana".to_string(),
                precio: 2500.0,
                opciones: Some(vec![
                    IOpciones {
                        modificador: "Extra 1".to_string(),
                        cantidad: 1,
                        opcion: "Mozzarella".to_string(),
                    },
                    IOpciones {
                        modificador: "Extra 2".to_string(),
                        cantidad: 1,
                        opcion: "Piña".to_string(),
                    },
                ]),
                comentario: Some("Sin aceitunas, por favor".to_string()),
            },
            Item {
                cantidad: 5.0,
                nombre: "πz²a".to_string(),
                precio: 2500.0,
                opciones: Some(vec![]),
                comentario: Some("".to_string()),
            },
        ],
        sub_total: 5000,
        gastos_envio: 1000,
        dscto_cupon_gasto_envio: 0.0,
        dscto_cupon_subtotal: 0.0,
        dscto_puntos: 0.0,
        cupones: None,
        fechas: Fechas {
            fecha_salida_cocina_estimada: "2024-12-25T14:30:00Z".to_string(),
            fecha_entrega_min: "2024-12-25T14:31:00Z".to_string(),
            fecha_pago: "2024-12-25T14:05:00Z".to_string(),
            tz: "America/Santiago".to_string(),
        },
        tipo_entrega: TipoEntrega { id: 1}, // 1 = Delivery, 2 = Retiro, etc.
        drop_off: Some(DropOff {
            tipo_entrega: Some("tipo entrega viene como string".to_string()),
            direccion: Some("Av Siemrpe Viva 420, titirilquen".to_string()),
        }),
        courier: Courier { id_courier: -2 }, // -2 = Reparto Propio, ejemplo
        cliente: Cliente {
            telefono: Some("+56 9 1234 5678".to_string()),
            nombre: Some("Juan Pérez".to_string()),
            nro: Some("Depto. 202".to_string()),
        },
        sucursal: None, // No aplica si es delivery
        plataforma: Plataforma {
            codigo: Some("AGIL".to_string()),
            nombre: Some("Agil".to_string()),
        },
        correlativo: 1001,
        codigo: "P42069".to_string(),
        comercio: Comercio {
            nombre: Some("La Pizzería".to_string()),
        },
        pago: Pago {
            medio_pago: MedioPago {
                nombre: Some("Tarjeta".to_string()),
            },
        },
    };

    pdf(&orden_ejemplo);
    // println!("espacio creado")
}

/*

 * Matias del futuro:
 *  recuerda que 
 *  refactorizando
 *  la creacion del largo de los tickets 
 *  para normalizar los tamaños
 * 
 * cambia los SET de textos cambien el alto final cuando lo realicen se un texto opcional
 * tambien el SET de parrafos cuando se extiendan
 * ademas de modificar los textos dependientes del cambio

*/

// ! Done:
//  dejarlo como el disclamer como original 


// ToDo:
// correlativo Tamaño Salida cocina (bold)
// saldida cocina subir a hora salida
// fondo blanco a los iconos mas anchos.
//      * realizado pero falta modifciar las iamgenes que sean de fonodo 1:1 y una imagen alta en 2:3 
//  mas espacios entre los cambios de segmenteo
//  comentarios de productos
//  comillas en cometarios
//  aumentar fuente comentarios
//  precio modificadores
//  fechas en latino

// descuentos ofertas :
//  * descuentos falsos
// descuentos codigos :
// - envio (cupon) 
// - cupon
// - puntos