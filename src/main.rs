extern crate printpdf;
use printpdf::*;
use std::{convert::From, os::linux::raw::stat};

mod pdf_resources;
use pdf_resources::{
    format_clp, format_datetime, set_linea_horizontal, PdfResources,
};

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

fn pdf(orden: &IOrder) {
    let mut pdf = PdfResources::new();
    let mut y_actual = 0.;
    // CUERPO 0: header

    // reimpreso:
    let reimpreso = String::from("*REIMPRESO*");
    pdf.set_paragraph(&reimpreso, 16.0, 5.0, 70.0, -1, false);

    // comercio nombre
    let comercio_nombre = orden.comercio.nombre.as_ref().unwrap();
    y_actual = pdf.set_paragraph(
        &comercio_nombre,
        20.0,
        y_actual + 12.0,
        70.0,
        0,
        false,
    );

    // plataforma nombre
    let plataforma_nombre = orden.plataforma.nombre.as_ref().unwrap();
    y_actual = pdf.set_paragraph(
        &plataforma_nombre,
        16.0,
        y_actual + 0.0,
        70.0,
        0,
        false,
    );
    // moto
    if orden.tipo_entrega.id == 1 {
        pdf.set_img(5.0, y_actual + 4.0, 10.0, 10.0, "moto");
    } else {
        pdf.set_img(
            5.0,
            y_actual + 4.0,
            10.0,
            10.0,
            "camino",
        );
    }
    // correlativo
    let correlativo_string = orden.correlativo.to_string();
    y_actual = pdf.set_paragraph(
        &correlativo_string,
        30.0,
        y_actual + 4.0,
        70.0,
        1,
        false,
    );

    // nuestro (si es reparto propio)
    if orden.courier.id_courier == -2 {
        let nuestro_string = String::from("*NUESTRO*");
        let nuestro: &String = &nuestro_string;
        pdf.set_paragraph(
            &nuestro,
            16.0,
            y_actual - 4.0,
            70.0,
            -1,
            false,
        );
    }
    // codigo pedido
    //       anteponer el # al codigo:
    let codigo_pedido = String::from("#") + &orden.codigo;
    pdf.set_paragraph(
        &codigo_pedido,
        16.0,
        y_actual - 4.0,
        70.0,
        1,
        true,
    );

    // hora salida cociona
    let salida_cocina =
        format_datetime(&orden.fechas.fecha_salida_cocina_estimada.as_ref());
    pdf.set_paragraph(
        &salida_cocina.1,
        32.0,
        y_actual + 5.0,
        70.0,
        1,
        false,
    );

    // salida cocina
    let salida_cocina_string = String::from("Salida Cocina");
    y_actual = pdf.set_paragraph(
        &salida_cocina_string,
        12.0,
        y_actual + 5.0,
        70.0,
        -1,
        true,
    );

    // Cliente nombre
    pdf.set_linea(y_actual - 4.0);
    let cliente_nombre = orden.cliente.nombre.as_ref().unwrap();
    y_actual = pdf.set_paragraph(
        &cliente_nombre,
        24.0,
        y_actual + 4.0,
        70.0,
        0,
        false,
    );
    // ubicacion
    pdf.set_separacion(y_actual - 8.0, "ubicacion");
    // si es delivery

    let direccion = if orden.tipo_entrega.id == 1 {
        orden.drop_off.as_ref().unwrap().direccion.as_ref().unwrap() as &String
    } else {
        orden.sucursal.as_ref().unwrap().nombre.as_ref().unwrap() as &String
    };
    y_actual = pdf.set_paragraph(
        &direccion,
        14.0,
        y_actual + 5.0,
        50.0,
        0,
        false,
    );
    let tipo_entrega = orden
        .drop_off
        .as_ref()
        .unwrap()
        .tipo_entrega
        .as_ref()
        .unwrap();
    y_actual = pdf.set_paragraph(
        &tipo_entrega,
        14.0,
        y_actual + 3.0,
        80.0,
        0,
        true,
    );

    // hora pago
    let static_hora_pago = String::from("Hora de Pago");
    pdf.set_paragraph(
        &static_hora_pago,
        14.0,
        y_actual + 2.0,
        70.0,
        -1,
        true,
    );
    let fecha_pago = format_datetime(&orden.fechas.fecha_pago.as_ref());
    let str_fecha_entrega = fecha_pago.0 + ". - " + &fecha_pago.1;
    y_actual = pdf.set_paragraph(
        &str_fecha_entrega,
        14.0,
        y_actual + 2.0,
        70.0,
        1,
        true,
    );
    // hora entrega
    let static_hora_entrega = String::from("Hora de Entrega");
    pdf.set_paragraph(
        &static_hora_entrega,
        14.0,
        y_actual + 1.0,
        70.0,
        -1,
        true,
    );
    let fecha_entrega =
        format_datetime(&orden.fechas.fecha_entrega_min.as_ref());
    let str_fecha_entrega = fecha_entrega.0 + ". - " + &fecha_entrega.1;
    y_actual = pdf.set_paragraph(
        &str_fecha_entrega,
        14.0,
        y_actual + 1.0,
        70.0,
        1,
        true,
    );
    let inicio_rect = y_actual - 3.0;
    // comentario cliente
    let static_comentario_cliente = String::from(" Comentario del Cliente: ");
    y_actual = pdf.set_paragraph(
        &static_comentario_cliente,
        14.0,
        y_actual + 2.0,
        60.0,
        -2,
        false,
    );
    // agregar comillas dobles al final e inicio del texto
    let comentario: String =
        " \"".to_string() + orden.comentario.as_ref().unwrap() + "\"";

    y_actual = pdf.set_paragraph(
        &comentario,
        16.0,
        y_actual + 1.0,
        68.0,
        0,
        true,
    );
    pdf.set_rect(inicio_rect, y_actual - 2.0);

    pdf.set_separacion(y_actual, "cubiertos");
    let mut precio_total = 0;
    y_actual += 5.0;
    // CUERPO 2: pedidos
    for item in &orden.items {
        pdf.set_paragraph(
            &(item.cantidad.to_string() + " X " + &item.nombre),
            13.0,
            y_actual + 5.0,
            70.0,
            -1,
            false,
        );
        let num_precio = (item.precio * item.cantidad) as i32;
        precio_total += num_precio;
        let precio = format_clp(num_precio);
        y_actual = pdf.set_paragraph(
            &precio,
            13.0,
            y_actual + 5.0,
            70.0,
            1,
            false,
        );
        for modi in item.opciones.as_ref().unwrap() {
            y_actual = pdf.set_paragraph(
                &("- ".to_string() + &modi.modificador),
                13.0,
                y_actual + 2.0,
                70.0,
                -2,
                true,
            );
            y_actual = pdf.set_paragraph(
                &("".to_string()
                    + &modi.cantidad.to_string()
                    + " X   "
                    + &modi.opcion),
                13.0,
                y_actual + 0.0,
                70.0,
                -2,
                true,
            );
        }
        if item.comentario.as_ref().unwrap() != "" {
            let ped_inicio_rect = y_actual - 3.0;
            // comentario cliente
            let static_comentario_cliente =
                String::from(" Comentario del Cliente: ");
            y_actual = pdf.set_paragraph(
                &static_comentario_cliente,
                14.0,
                y_actual + 2.0,
                60.0,
                -2,
                false,
            );
            // agregar comillas dobles al final e inicio del texto
            let ped_comentario: String =
                " \"".to_string() + item.comentario.as_ref().unwrap() + "\"";
            y_actual = pdf.set_paragraph(
                &ped_comentario,
                16.0,
                y_actual + 1.0,
                68.0,
                0,
                true,
            );
            pdf.set_rect(ped_inicio_rect, y_actual - 2.0);
        }
    }

    // FOOTER: pagos
    pdf.set_separacion(y_actual, "dinero");

    let descuento_monto: (f32, bool, String) = // bool es si es cupon de gasto envio o no
        if orden.dscto_cupon_gasto_envio > 0.0 {
            (
                orden.dscto_cupon_gasto_envio,
                true,
                "Descuento (".to_string()
                    + &orden.cupones.as_ref().unwrap()[0].codigo
                    + ")",
            )
        } else if orden.dscto_cupon_subtotal > 0.0 {
            (
                orden.dscto_cupon_subtotal,
                false,
                "Descuento".to_string(),
            )
        } else {
            (
                orden.dscto_puntos,
                false,
                "Puntos".to_string(),
            )
        };

    let descuento_oferta = orden.sub_total - precio_total;
    let gastos_envio = orden.gastos_envio;
    let total = orden.sub_total + orden.gastos_envio + descuento_monto.0 as i32;

    y_actual += 8.0;
    pdf.set_paragraph(
        &String::from("Subtotal"),
        16.0,
        y_actual + 2.0,
        70.0,
        -1,
        true,
    );
    let precio_subtotal = format_clp(orden.sub_total);
    y_actual = pdf.set_paragraph(
        &precio_subtotal,
        16.0,
        y_actual + 2.0,
        70.0,
        1,
        false,
    );

    if descuento_oferta > 0 {
        pdf.set_paragraph(
            &String::from("Descuento Oferta"),
            16.0,
            y_actual + 1.0,
            70.0,
            -1,
            true,
        );
        let precio_descuento_oferta = format_clp(descuento_oferta);
        y_actual = pdf.set_paragraph(
            &precio_descuento_oferta,
            16.0,
            y_actual + 1.0,
            70.0,
            1,
            false,
        );
    }
    if descuento_monto.0 > 0.0 && !descuento_monto.1 {
        pdf.set_paragraph(
            &descuento_monto.2,
            16.0,
            y_actual + 1.0,
            70.0,
            -1,
            true,
        );
        let precio_descuento_monto = format_clp(descuento_monto.0 as i32);
        y_actual = pdf.set_paragraph(
            &precio_descuento_monto,
            16.0,
            y_actual + 1.0,
            70.0,
            1,
            false,
        );
    }
    if gastos_envio > 0 {
        pdf.set_paragraph(
            &String::from("Despacho"),
            16.0,
            y_actual + 1.0,
            70.0,
            -1,
            true,
        );
        let precio_gastos_envio = format_clp(gastos_envio);
        y_actual = pdf.set_paragraph(
            &precio_gastos_envio,
            16.0,
            y_actual + 1.0,
            70.0,
            1,
            false,
        );
    }
    if descuento_monto.0 > 0.0 && descuento_monto.1 {
        pdf.set_paragraph(
            &descuento_monto.2,
            16.0,
            y_actual + 1.0,
            70.0,
            -1,
            true,
        );
        let precio_descuento_monto = format_clp(descuento_monto.0 as i32);
        y_actual = pdf.set_paragraph(
            &precio_descuento_monto,
            16.0,
            y_actual + 1.0,
            70.0,
            1,
            false,
        );
    }
    pdf.set_paragraph(
        &String::from("Total"),
        16.0,
        y_actual + 1.0,
        70.0,
        -1,
        true,
    );
    let precio_total = format_clp(total);
    y_actual = pdf.set_paragraph(
        &precio_total,
        16.0,
        y_actual + 1.0,
        70.0,
        1,
        false,
    );
    // disclaimer
    let disclaimer =
        String::from("* Total no incluye propina ni cuota de servicio.");
    y_actual = pdf.set_paragraph(
        &disclaimer,
        9.0,
        y_actual + 1.0,
        70.0,
        -1,
        true,
    );

    // medio de pago
    let medio_pago = orden.pago.medio_pago.nombre.as_ref().unwrap();
    y_actual = pdf.set_paragraph(
        &medio_pago,
        16.0,
        y_actual + 5.0,
        70.0,
        1,
        false,
    );

    let power_agil = String::from("powered by Agil");
    y_actual = pdf.set_paragraph(
        &power_agil,
        12.0,
        y_actual + 2.0,
        80.0,
        0,
        true,
    );

    // pdf.set_rect(10.0, 20.0);
    pdf.init_draw();
    pdf.drow_all_obj();
    pdf.save_pdf();

    /*


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

        let copia_string = String::from("*REIMPRESO*");
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

    // // Cost subtotal
    // set_texto(
    //     &current_layer,
    //     16.0,
    //     &String::from("Subtotal"),
    //     44.0,
    //     &resources,
    //     -1,
    //     true,
    // );
    // let costo_total = format_clp(orden.sub_total as i32);
    // set_texto(
    //     &current_layer,
    //     16.0,
    //     &costo_total,
    //     44.0,
    //     &resources,
    //     1,
    //     false,
    // );
    // set_separacion(&resources, &current_layer, 50.0, "dinero");
    // if orden.tipo_entrega.id == 1 {
    //     gastos_envio = orden.gastos_envio;
    //     let str_gastos_envio = format_clp(gastos_envio);
    //     set_texto(
    //         &current_layer,
    //         16.0,
    //         &String::from("Despacho"),
    //         37.0,
    //         &resources,
    //         -1,
    //         true,
    //     );
    //     set_texto(
    //         &current_layer,
    //         16.0,
    //         &str_gastos_envio,
    //         37.0,
    //         &resources,
    //         1,
    //         false,
    //     );
    // }
    // // costo total
    // set_texto(
    //     &current_layer,
    //     16.0,
    //     &String::from("Total"),
    //     30.0,
    //     &resources,
    //     -1,
    //     true,
    // );
    // let costo_total = format_clp(orden.sub_total + gastos_envio + descuento_monto);
    // set_texto(
    //     &current_layer,
    //     16.0,
    //     &costo_total,
    //     30.0,
    //     &resources,
    //     1,
    //     false,
    // );

    // // no incluye ...
    // let disclaimer = String::from("* Total no incluye propina ni cuota de servicio.");
    // set_texto(
    //     &current_layer,
    //     9.0,
    //     &disclaimer,
    //     24.0,
    //     &resources,
    //     -1,
    //     true,
    // );

    // // medio de pago ...
    // let medio_pago = orden.pago.medio_pago.nombre.as_ref().unwrap();
    // set_texto(
    //     &current_layer,
    //     20.0,
    //     medio_pago,
    //     17.0,
    //     &resources,
    //     1,
    //     false,
    // );

    // power by agil

    */
}

fn main() {
    let orden_ejemplo = IOrder {
        comentario: Some("Orden de ejemplo".to_string()),
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
                comentario: Some("Sin aceitunas, por favor por favor por favor por favor por favor por favor!!!".to_string()),
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
        tipo_entrega: TipoEntrega { id: 1 }, // 1 = Delivery, 2 = Retiro, etc.
        drop_off: Some(DropOff {
            tipo_entrega: Some("tipo entrega viene como string".to_string()),
            direccion: Some("Av Siemrpe Viva 420, titirilquen".to_string()),
        }),
        courier: Courier { id_courier: -2 }, // -2 = Reparto Propio, ejemplo
        cliente: Cliente {
            telefono: Some("+56 9 1234 5678".to_string()),
            nombre: Some("Pablo Diego José Francisco de Paula Juan Nepomuceno María de los Remedios Cipriano de la Santísima Trinidad Ruiz y Picasso".to_string()),
            nro: Some("Depto. 202".to_string()),
        },
        sucursal: Some(Sucursal {
            nombre: Some("algun lado".to_string()), // No aplica si es delivery
        }),
        plataforma: Plataforma {
            codigo: Some("AGIL".to_string()),
            nombre: Some("Agil".to_string()),
        },
        correlativo: 9,
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
