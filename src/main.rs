use printpdf::path::{PaintMode, WindingOrder};
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::convert::From;
use ::image::ImageReader;
use ::image::{DynamicImage, ImageError};
use printpdf::Image;

use ttf_parser::Face;

// Convierte al tipo Image de printpdf
pub struct IOrder {
    pub comentario: Option<String>,
    pub items: Vec<Item>,
    pub sub_total: f64,
    pub gastos_envio: f64,
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
    texto(
        &current_layer,
        20.0,
        orden.cliente.telefono.as_ref().unwrap(),
        68.0,
        &resources,
        0,
        false,
    );

    line_vert(&current_layer, 70.0, &resources);
    line_vert(&current_layer, 72.0, &resources);

    // ubicacion
    let ubicacion: String = orden
        .drop_off
        .as_ref()
        .unwrap()
        .direccion
        .clone() // Asegura que dirección es una opción de copia
        .unwrap_or("".to_string());
    texto(&current_layer, 14.0, &ubicacion, 77.0, &resources, 0, false);

    // nuestro
    let nuestro_string = String::from("NUESTRO");
    let nuestro: &String = &nuestro_string;
    texto(&current_layer, 18.0, nuestro, 45.0, &resources, -1, false);

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
    let hora_salida_cocina_string = String::from("13:56"); // TODO
    let hora_salida_cocina: &String = &hora_salida_cocina_string;
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
    let mut buffer: BufWriter<File> = BufWriter::new(File::create("test_working.pdf").unwrap());
    resources.doc.save(&mut buffer).unwrap();
}

fn line_vert(current_layer: &PdfLayerReference, altura: f64, resources: &PdfResources) {
    // let i:f32  = (resources.page_height - altura) as f32;
    let i = resources.page_height - altura;
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


fn imagen(
    current_layer: &PdfLayerReference,

) {
    // Abre el archivo de imagen PNG
    fn cargar_imagen(
        image_path: &str
    ) -> Result<DynamicImage, ImageError> {
        let img: DynamicImage = ImageReader::open(image_path)?.decode()?;
        Ok(img)
    }

    let image_path = "assets/img/Icon_Moto.png";
    let img: Result<DynamicImage, ImageError> = cargar_imagen(image_path);
    // Convierte al tipo Image de printpdf    
  
    // aqui quede
    // aqui quede
    // aqui quede
    // aqui quede
    // aqui quede    
    let image = Image::try_from(Image::codecs::bmp::BmpDecoder::new(&mut image_file).unwrap()).unwrap();
    // aqui quede
    // aqui quede
    // aqui quede
    // aqui quede
    // aqui quede
/*
use printpdf::*;

fn main() {
    let mut doc = PdfDocument::new("My first PDF");
    let image_bytes = include_bytes!("assets/img/dog_alpha.png");
    let image = RawImage::decode_from_bytes(image_bytes).unwrap(); // requires --feature bmp

    // In the PDF, an image is an `XObject`, identified by a unique `ImageId`
    let image_xobject_id = doc.add_image(&image);

    let page1_contents = vec![Op::UseXObject {
        id: image_xobject_id.clone(),
        transform: XObjectTransform::default(),
    }];

    let page1 = PdfPage::new(Mm(210.0), Mm(297.0), page1_contents);
    let pdf_bytes: Vec<u8> = doc.with_pages(vec![page1]).save(&PdfSaveOptions::default());
    let _ = std::fs::write("image.pdf", pdf_bytes);
}
*/

    // image.add_to_layer()
   
}
fn texto(use printpdf::*;

fn main() {
    let mut doc = PdfDocument::new("My first PDF");
    let image_bytes = include_bytes!("assets/img/dog_alpha.png");
    let image = RawImage::decode_from_bytes(image_bytes).unwrap(); // requires --feature bmp

    // In the PDF, an image is an `XObject`, identified by a unique `ImageId`
    let image_xobject_id = doc.add_image(&image);

    let page1_contents = vec![Op::UseXObject {
        id: image_xobject_id.clone(),
        transform: XObjectTransform::default(),
    }];

    let page1 = PdfPage::new(Mm(210.0), Mm(297.0), page1_contents);
    let pdf_bytes: Vec<u8> = doc.with_pages(vec![page1]).save(&PdfSaveOptions::default());
    let _ = std::fs::write("image.pdf", pdf_bytes);
}
    current_layer: &PdfLayerReference,
    font_size: f64,
    text: &String,
    altura: f64,
    resources: &PdfResources,
    tipo: i8,
    light: bool,
) {
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
    let y_position: f64 = resources.page_height - altura;
    println!("{}", text);

    current_layer.use_text(
        text,
        font_size as f32,
        Mm(x_position as f32),
        Mm(y_position as f32),
        &font_use,
    );
}

fn main() {
    let orden_ejemplo = IOrder {
        comentario: Some("Orden de ejemplo".to_string()),
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
        sub_total: 5000.0,
        gastos_envio: 1000.0,
        dscto_cupon_gasto_envio: 0.0,
        dscto_cupon_subtotal: 0.0,
        dscto_puntos: 0.0,
        cupones: None,
        fechas: Fechas {
            fecha_salida_cocina_estimada: "2024-12-25T14:30:00Z".to_string(),
            tz: "America/Santiago".to_string(),
            fecha_pago: "2024-12-25T14:05:00Z".to_string(),
        },
        tipo_entrega: TipoEntrega { id: 1 }, // 1 = Delivery, 2 = Retiro, etc.
        drop_off: Some(DropOff {
            tipo_entrega: Some("Delivery".to_string()),
            direccion: Some("Calle Falsa 123".to_string()),
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
    println!("espacio creado")
}
