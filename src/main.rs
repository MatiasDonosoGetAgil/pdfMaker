
// use printpdf::Image;
// use ::image::png::PngDecoder;
// use ::image::{ImageDecoder, ImageDecoderExt, ImageFormat};
// use printpdf::path::{PaintMode, WindingOrder};
// use ::image::io::Reader as ImageReader;
// Si necesitas redimensionar la imagen para que coincida con width y height:

// use ::image::GenericImageView;
// use ::image::{ColorType, ImageDecoder};
// use ::image::codecs::png::PngDecoder;
// use printpdf::Image;

// winding_order: WindingOrder::NonZero,
// mode: PaintMode::FillStroke,
extern crate printpdf;
use printpdf::*;

use printpdf::path::{PaintMode, WindingOrder};
use std::convert::From;
use std::default;
use std::fs::File;
use std::io::BufWriter;
use std::mem::transmute;

use ttf_parser::{Face, Transform};
use chrono::prelude::*;


// Convierte al tipo Image de printpdf
pub struct IOrder {
    pub comentario: Option<String>,
    pub items: Vec<Item>,
    pub dscto_cupon_gasto_envio: f64,
    pub dscto_cupon_subtotal: f64,
    pub dscto_puntos: f64,
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
    pub sub_total: u64,
    pub gastos_envio: u64,
}

/// Representa un producto o ítem dentro de la orden.
pub struct Item {
    pub cantidad: f64, // Equivalente a `number` en TS
    pub nombre: String,
    pub precio: f64,
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
    upem: f64,
    font_light: IndirectFontRef,
    face_light: Face<'a>,
    upem_light: f64,
    page_width: f64,
    page_height: f64,
    doc: PdfDocumentReference,
    page: PdfPageIndex,
    layer: PdfLayerIndex,
}

impl<'a> PdfResources<'a> {
    fn new() -> Self {
        let page_width = 80.0;
        let page_height = 190.0;
        let (doc, page, layer) =
            PdfDocument::new("Ticket", Mm(page_width as f32), Mm(page_height as f32), "");

        let font_data: &[u8] = include_bytes!("../assets/fonts/Roboto-Black.ttf") as &[u8];
        let face: Face<'_> = Face::parse(font_data, 0).expect("No se pudo cargar la fuente");
        let upem: f64 = face.units_per_em() as f64;
        let font: IndirectFontRef = doc.add_external_font(font_data).expect(
            "Failed to add external font. Ensure the font data is correct and file path is valid.",
        );

        let font_ligth_data: &[u8] =
            include_bytes!("../assets/fonts/Roboto-Regular.ttf") as &[u8];
        let face_light: Face<'_> =
            Face::parse(font_ligth_data, 0).expect("No se pudo cargar la fuente");
        let upem_light: f64 = face_light.units_per_em() as f64;
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
    let resources = PdfResources::new();

    let current_layer: PdfLayerReference = resources
        .doc
        .get_page(resources.page)
        .get_layer(resources.layer);

    // In the PDF, an image is an `XObject`, identified by a unique `ImageId`
    let mut extra = 0.0;
    // comercio nombre
    texto(
        &current_layer,
        24.0,
        orden.comercio.nombre.as_ref().unwrap(),
        14.0,
        &resources,
        0,
        false,
    );

    // plataforma nombre
    texto(
        &current_layer,
        16.0,
        orden.plataforma.nombre.as_ref().unwrap(),
        19.0,
        &resources,
        0,
        false,
    );

    line_vert(&current_layer, 52.0, &resources);

    // Cliente nombre
    texto(
        &current_layer,
        24.0,
        orden.cliente.nombre.as_ref().unwrap(),
        60.0,
        &resources,
        0,
        false,
    );

    // Cliente Numero
    if orden.courier.id_courier == -2 {
        texto(
            &current_layer,
            20.0,
            orden.cliente.telefono.as_ref().unwrap(),
            68.0,
            &resources,
            0,
            false,
        );

        // nuestro
        let nuestro_string = String::from("NUESTRO");
        let nuestro: &String = &nuestro_string;
        texto(&current_layer, 18.0, nuestro, 45.0, &resources, -1, false);
        extra += 8.0
    }

    line_vert(&current_layer, 62.0 + extra, &resources);
    line_vert(&current_layer, 62.75 + extra, &resources);

    // numero pedido
    let numero_pedido_string = String::from("#P123456");
    let numero_pedido: &String = &numero_pedido_string;
    texto(
        &current_layer,
        14.0,
        numero_pedido,
        38.0,
        &resources,
        1,
        true,
    );

    // salida_cocina
    let salida_cocina_string = String::from("Salida Cocina");
    let salida_cocina: &String = &salida_cocina_string;
    texto(
        &current_layer,
        12.0,
        salida_cocina,
        50.0,
        &resources,
        -1,
        true,
    );

    // hora salida cociona
    let salida_cocina = format_datetime(orden.fechas.fecha_salida_cocina_estimada.as_ref());
    let hora_salida_cocina: &String = &salida_cocina.1;
    texto(
        &current_layer,
        32.0,
        hora_salida_cocina,
        48.0,
        &resources,
        1,
        false,
    );

    // copia ?

    let copia_string = String::from("REIMPRESO"); // TODO
    let copia: &String = &copia_string;
    texto(&current_layer, 12.0, copia, 5.0, &resources, -1, false);

    // ubicacion
    let mut ubicacion: String = orden
        .drop_off
        .as_ref()
        .unwrap()
        .direccion
        .clone() // Asegura que dirección es una opción de copia
        .unwrap_or("".to_string());
    ubicacion = format!("   {}", ubicacion);

    let formatted_ubicacion = format!("  {}", ubicacion);
    let ubicacion_co: (f64, f64) = texto(
        &current_layer,
        14.0,
        &formatted_ubicacion,
        69.0 + extra,
        &resources,
        0,
        false,
    );
    // icono ubicacion
    let _ = imagen(
        &current_layer,
        &resources,
        ubicacion_co.0 as f32,
        69.0 + extra as f32,
        4.0,
        4.0,
        "assets/img/Icon_Moto.png",
    );
    // indicacion
    let tipo: String = orden
        .drop_off
        .as_ref()
        .unwrap()
        .tipo_entrega
        .clone()
        .unwrap_or("".to_string());

    texto(
        &current_layer,
        14.0,
        &tipo,
        74.0 + extra,
        &resources,
        0,
        true,
    );

    // hora pago
    texto(
        &current_layer,
        14.0,
        &String::from("Hora de Pago"),
        82.0 + extra,
        &resources,
        -1,
        true,
    );

    let fecha_pago = format_datetime(&orden.fechas.fecha_pago.as_ref());
    let hora_pago = format!("{}. - {}", fecha_pago.0, fecha_pago.1);    
    texto(
        &current_layer,
        12.0,
        &hora_pago,
        82.0 + extra,
        &resources,
        1,
        true,
    );

    // hora entrega
    texto(
        &current_layer,
        14.0,
        &String::from("Hora de Entrega"),
        87.0 + extra,
        &resources,
        -1,
        true,
    );

    let fecha_entrega = format_datetime(&orden.fechas.fecha_entrega_min.as_ref());
    let hora_entrega = format!("{}. - {}", fecha_entrega.0, fecha_entrega.1);  
    texto(
        &current_layer,
        12.0,
        &hora_entrega,
        87.0 + extra,
        &resources,
        1,
        true,
    );

    //   altura  = 95.0,
    /*
     Calculo relativo al inicio pero de tamaño variable!!!
    */
    // Comentario cliente
    line_vert(&current_layer, 89.0 + extra, &resources);
    texto(
        &current_layer,
        12.0,
        &String::from(" Comentario del Cliente: "),
        94.0 + extra,
        &resources,
        -1,
        false,
    );

    let comentario: &String = orden.comentario.as_ref().unwrap();

    let comenetario_co = parrafo(
        &current_layer,
        10.0,
        &comentario,
        99.0 + extra,
        &resources,
        -1,
        true,
    );

    let altura_nueva = resources.page_height - comenetario_co.1 + extra;
    lines_hor(&current_layer, 89.0 + extra, altura_nueva, &resources);
    line_vert(&current_layer, altura_nueva, &resources);

    // cubiertos
    line_vert(&current_layer, altura_nueva + 5.0, &resources);
    line_vert(&current_layer, altura_nueva + 5.75, &resources);
    let cubiertos = altura_nueva + 9.0;
    let _ = imagen(&current_layer, &resources, 37.5, cubiertos as f32, 6.0, 8.0, "assets/img/Icon_Moto.png");
    // productos
    // modificadores

    /*
     Calculo relativo al final
    */

    line_vert(&current_layer, -55.0, &resources);

    // Cost subtotal
    texto(
        &current_layer,
        18.0,
        &String::from("Subtotal"),
        -48.0,
        &resources,
        -1,
        true,
    );
    let costo_total = format_to_chilean_money(orden.sub_total);
    texto(
        &current_layer,
        18.0,
        &costo_total,
        -48.0,
        &resources,
        1,
        false,
    );
    // Costo despacho
    texto(
        &current_layer,
        18.0,
        &String::from("Despacho"),
        -40.0,
        &resources,
        -1,
        true,
    );
    let costo_total = format_to_chilean_money(orden.gastos_envio);
    texto(
        &current_layer,
        18.0,
        &costo_total,
        -40.0,
        &resources,
        1,
        false,
    );
    // costo total
    texto(
        &current_layer,
        18.0,
        &String::from("Total"),
        -32.0,
        &resources,
        -1,
        true,
    );
    let costo_total = format_to_chilean_money(orden.sub_total + orden.gastos_envio);
    texto(
        &current_layer,
        18.0,
        &costo_total,
        -32.0,
        &resources,
        1,
        false,
    );

    // no incluye ...
    let disclaimer = String::from("* Total no incluye propina ");
    texto(
        &current_layer,
        9.0,
        &disclaimer,
        -27.0,
        &resources,
        -1,
        true,
    );
    let disclaimer2 = String::from("ni cuota de servicio.");
    texto(
        &current_layer,
        9.0,
        &disclaimer2,
        -24.0,
        &resources,
        -1,
        true,
    );
    // medio de pago ...
    let medio_pago = orden.pago.medio_pago.nombre.as_ref().unwrap();
    texto(
        &current_layer,
        20.0,
        medio_pago,
        -17.0,
        &resources,
        1,
        false,
    );

    // power by agil
    let power_agil = String::from("powered by Agil");
    texto(
        &current_layer,
        12.0,
        &power_agil,
        -10.0,
        &resources,
        0,
        true,
    );

    // Envio
    let _ = imagen(&current_layer, &resources, 5.0, 30.0, 12.0, 10.0, "assets/img/Icon_Moto.png");

    let mut buffer: BufWriter<File> = BufWriter::new(File::create("test_working.pdf").unwrap());

    resources.doc.save(&mut buffer).unwrap();
}


fn format_datetime(iso_date: &str) -> (String, String) {
    // Parseamos la fecha ISO
    let datetime = DateTime::parse_from_rfc3339(iso_date)
        .expect("Error parsing ISO date");
    
    // Formateamos la fecha como "DD MMM"
    let date = datetime.format("%d %b").to_string();
    
    // Formateamos la hora como "HH:MM"
    let time = datetime.format("%H:%M").to_string();
    
    (date, time)
}


fn format_to_chilean_money(amount: u64) -> String {
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

fn line_vert(current_layer: &PdfLayerReference, altura: f64, resources: &PdfResources) {
    // let i:f32  = (resources.page_height - altura) as f32;
    let mut i = -altura;
    if i < 0.0 {
        i = resources.page_height - altura;
    }
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

fn lines_hor(current_layer: &PdfLayerReference, inicio: f64, fin: f64, resources: &PdfResources) {
    let y_inicio = (resources.page_height - inicio) as f32;
    let y_fin = (resources.page_height - fin) as f32;
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

fn imagen(
    current_layer: &PdfLayerReference,
    resources: &PdfResources,
    x: f32,
    y: f32,
    ancho: f64,
    alto: f64,
    path_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fn mm_to_px(mm: f64, dpi: f64) -> usize {
        // There are 25.4 millimeters in an inch.
        let inches = mm / 25.4;
        // Convert inches to pixels using the specified DPI
        let pixels = inches * dpi;
        pixels.round() as usize
    }

    // Aumentamos los DPI a 600
    const DPI: f64 = 600.0;
    let width: Px = Px(mm_to_px(ancho, DPI));
    let height: Px = Px(mm_to_px(alto, DPI));
    
    // Cargar la imagen PNG
    // let image = Image::from_dynamic_image(image::open("ruta_a_imagen.bmp").unwrap());
    let img: Image = Image::from_dynamic_image(&image_crate::open(path_name).unwrap());
    // println!("{}", path_name);
    // println!("Dimensiones originales: {}x{}", img.width(), img.height());
    // println!("Dimensiones objetivo: {}x{}", width.0, height.0);
    

    // let gray_img = img.into_luma8();
    
    // Crear un nuevo vector con fondo blanco
    // let mut image_data = vec![255u8; width.0 * height.0]; // Inicializar con blanco (255)
    let mut image_data: Vec<u8> = vec![0u8; width.0 * height.0]; // Inicializar con negro (255)
        // Antes de extraer el raw data, guardamos lo necesario.
    // let (_img_width, img_height) = (gray_img.width(), gray_img.height());
    // // Copiar los datos de la imagen original
    // let original_data = gray_img.into_raw();
    // let min_width = std::cmp::min(width.0, original_data.len() / img_height as usize);
    // let min_height = std::cmp::min(height.0, img_height as usize);

    // Copiar los datos manteniendo el fondo blanco
    for y in 0..min_height {
        for x in 0..min_width {
            let src_idx = y * (original_data.len() / img_height as usize) + x;
            let dst_idx = y * width.0 + x;
            if dst_idx < image_data.len() && src_idx < original_data.len() {
                image_data[dst_idx] = original_data[src_idx];
            }
        }
    }
    let image_x_object: ImageXObject = ImageXObject {
        width,
        height,
        color_space: ColorSpace::Greyscale,
        bits_per_component: ColorBits::Bit1,
        interpolate: true,  // Cambiado a true para mejor calidad
        image_data,
        image_filter: None,
        clipping_bbox: None,
        smask: None,
    };
    let image_transform: ImageTransform = ImageTransform {
        translate_x: Some(Mm(x)),
        translate_y: Some(Mm(resources.page_height as f32 - y)),
        rotate: None,
        scale_x: None,
        scale_y: None,
        dpi: Some(DPI as f32), // Usar los nuevos DPI aquí también
    };
    // Convierte el ImageXObject a un printpdf::Image
    // let image = Image::from(img);

    img.add_to_layer(current_layer.clone(), ImageTransform::default());
    
    Ok(())
}

/// Dibuja texto en forma de párrafo, con máximo 70mm de ancho por línea.
fn parrafo(
    current_layer: &PdfLayerReference,
    font_size: f64,
    text: &str,
    altura: f64,
    resources: &PdfResources,
    tipo: i8,
    light: bool,
) -> (f64, f64) {
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
    let measure_word_mm = |w: &str| -> f64 {
        let width_points: f64 = w
            .chars()
            .filter_map(|c| face_use.glyph_index(c))
            .map(|glyph_id| face_use.glyph_hor_advance(glyph_id).unwrap_or(0) as f64)
            .sum::<f64>()
            * scale_factor;

        // Convertir de puntos tipográficos a mm
        width_points * 0.352778
    };

    // 6. Construir “líneas” respetando el max_width_mm
    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();
    let mut current_line_width: f64 = 0.0;

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
    let line_spacing: f64 = font_size * 0.4; // Ajusta a tu gusto
    let mut y_position: f64 = if altura > 0.0 {
        // lo manejabas como: y_position = page_height - altura
        resources.page_height - altura
    } else {
        -altura
    };

    // Vamos a devolver la última posición X e Y dibujada.
    // El “último X” no es tan relevante porque cada línea
    // puede tener un X distinto. Si quieres solo dejarlo
    // en 10.0 o en el calculado por la última línea,
    // eso depende de tu uso.
    let mut last_x_position: f64 = 10.0;
    // let mut last_altura = altura;
    
    for line in &lines {
        // Medir el ancho de la línea para alinear:
        let line_width_mm = measure_word_mm(line);

        let x_position = if tipo < 0 {
            // Izquierda
            10.0
        } else if tipo == 0 {
            // Centrado
            (resources.page_width - line_width_mm) / 2.0
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

fn texto(
    current_layer: &PdfLayerReference,
    font_size: f64,
    text: &String,
    altura: f64,
    resources: &PdfResources,
    tipo: i8,
    light: bool,
) -> (f64, f64) {
    let mut font_use = resources.font.clone();
    let mut face_use = resources.face.clone();
    let mut upem_use = resources.upem.clone();
    if light {
        font_use = resources.font_light.clone();
        face_use = resources.face_light.clone();
        upem_use = resources.upem_light.clone();
    }

    let scale_factor = font_size / upem_use;

    let text_width_points: f64 = text
        .chars()
        .filter_map(|c| face_use.glyph_index(c))
        .map(|glyph_id| face_use.glyph_hor_advance(glyph_id).unwrap_or(0) as f64)
        .sum::<f64>()
        * scale_factor;

    let text_width_mm: f64 = text_width_points * 0.352778;

    let mut x_position: f64 = 5.0; // left
    if tipo == 0 {
        x_position = (resources.page_width - text_width_mm) / 2.0;
    } else if tipo > 0 {
        // right
        x_position = 75.0 - text_width_mm;
    }

    let mut y_position: f32 = 5.0;
    if altura > 0.0 {
        y_position =(resources.page_height - altura) as f32;
    } else {
        y_position = -altura as f32;
    }
    // println!("{}", text);

    current_layer.use_text(
        text,
        font_size as f32,
        Mm(x_position as f32),
        Mm(y_position as f32),
        &font_use,
    );
    (x_position, y_position.into())
}

fn main() {
    let orden_ejemplo = IOrder {
        comentario: Some("Orden de ejemplo ".to_string()),
        items: vec![Item {
            cantidad: 2.0,
            nombre: "Pizza Napolitana".to_string(),
            precio: 2500.0,
            opciones: Some(vec![
                IOpciones {
                    modificador: "Extra queso".to_string(),
                    cantidad: 1,
                    opcion: "Mozzarella".to_string(),
                },
                IOpciones {
                    modificador: "Sin cebolla".to_string(),
                    cantidad: 1,
                    opcion: "Quitar cebolla".to_string(),
                },
            ]),
            comentario: Some("Sin aceitunas, por favor".to_string()),
        }],
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
        tipo_entrega: TipoEntrega { id: 1 }, // 1 = Delivery, 2 = Retiro, etc.
        drop_off: Some(DropOff {
            tipo_entrega: Some("Delivery".to_string()),
            direccion: Some("Calle Falsa 123".to_string()),
        }),
        courier: Courier { id_courier: -1 }, // -2 = Reparto Propio, ejemplo
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
        codigo: "ORD-999".to_string(),
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


// fn main2() {
//     let mut doc = PdfDocument::new("My first PDF");
//     let image_bytes = include_bytes!("../assets/img/Icon_Moto.png");
//     let image = RawImage::decode_from_bytes(image_bytes).unwrap(); // requires --feature bmp

//     // In the PDF, an image is an `XObject`, identified by a unique `ImageId`
//     let image_xobject_id = doc.add_image(&image);

//     let page1_contents = vec![Op::UseXObject {
//         id: image_xobject_id.clone(),
//         transform: XObjectTransform::default(),
//     }];

//     let page1 = PdfPage::new(Mm(210.0), Mm(297.0), page1_contents);
//     let pdf_bytes: Vec<u8> = doc.with_pages(vec![page1]).save(&PdfSaveOptions::default());
//     let _ = std::fs::write("image.pdf", pdf_bytes);
// }
